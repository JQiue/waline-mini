use helpers::{
  hash, jwt,
  time::utc_now,
  uuid::{self, Alphabet},
};
use regex::Regex;
use sea_orm::{
  ColumnTrait, EntityTrait, IntoActiveModel, Iterable, PaginatorTrait, QueryFilter, QuerySelect,
  Set,
};
use serde_json::{Value, json};
use totp_rs::{Secret, TOTP};

use crate::{
  app::AppState,
  components::user::model::{SetUserProfileBody, UserLoginBody, UserRegisterBody},
  config::EnvConfig,
  entities::*,
  helpers::{
    avatar::get_avatar,
    email::{Notification, NotifyType, send_email_notification},
  },
  prelude::AppError,
  types::ServiceResult,
};

pub async fn user_register(
  state: &AppState,
  body: UserRegisterBody,
  host_header: String,
  lang: &str,
) -> ServiceResult<Value> {
  let UserRegisterBody {
    display_name,
    email,
    password,
    url,
  } = body;

  let EnvConfig {
    smtp_host,
    smtp_service,
    ..
  } = EnvConfig::load_env()?;

  let has_email_service = if smtp_host.is_some() || smtp_service.is_some() {
    true
  } else {
    false
  };
  let token = uuid::nanoid(&Alphabet::NUMBERS, 4);
  let mut normal_user_type = if has_email_service {
    format!(
      "verify:{}:{}",
      token,
      utc_now().timestamp_millis() + 60 * 60 * 1000
    )
  } else {
    "guest".to_string()
  };

  println!("{}", normal_user_type);

  let hashed = hash::bcrypt_custom(&password, 8, helpers::hash::Version::TwoA)?;

  if let Some(user) = state.repo.user().get_user_by_email(&email).await? {
    if user.user_type == "administrator" || user.user_type == "guest" {
      return Err(AppError::UserRegistered);
    }
    let mut active_user = user.into_active_model();
    active_user.display_name = Set(display_name);
    active_user.url = Set(Some(url));
    active_user.password = Set(hashed);
    active_user.user_type = Set(normal_user_type.clone());
    let url = format!("http://{host_header}/api/verification?token={token}&email={email}",);
    send_email_notification(Notification {
      sender_name: state.site_name.clone(),
      sender_email: email,
      comment_id: 0,
      comment: "".to_string(),
      url,
      notify_type: NotifyType::RegisterUser,
      lang: Some(lang),
    });
    state.repo.user().update_user(active_user).await?;
    if normal_user_type.starts_with("verify:") {
      return Ok(json!({
        "verify": true
      }));
    }
    Ok(json!({}))
  } else {
    let mut active_user: wl_users::ActiveModel = wl_users::ActiveModel {
      display_name: Set(display_name),
      email: Set(email.clone()),
      url: Set(Some(url)),
      password: Set(hashed),
      ..Default::default()
    };

    if state.repo.user().is_first_user().await? {
      normal_user_type = "guess".to_string();
      active_user.user_type = Set("administrator".to_string());
    } else {
      let token = uuid::nanoid(&Alphabet::NUMBERS, 4);
      active_user.user_type = Set(normal_user_type.clone());
      let url = format!("http://{host_header}/api/verification?token={token}&email={email}",);
      send_email_notification(Notification {
        sender_name: state.site_name.clone(),
        sender_email: email,
        comment_id: 0,
        comment: "".to_string(),
        url,
        notify_type: NotifyType::RegisterUser,
        lang: Some(lang),
      });
    }
    state.repo.user().create_user(active_user).await?;
    if normal_user_type.starts_with("verify:") {
      return Ok(json!({"verify": true}));
    }
    Ok(json!({}))
  }
}

pub async fn user_login(state: &AppState, body: UserLoginBody) -> ServiceResult<Value> {
  let UserLoginBody {
    code,
    email,
    password,
  } = body;
  let user = state
    .repo
    .user()
    .get_user_by_email(&email)
    .await?
    .ok_or(AppError::UserNotFound)?;
  if user.user_type.contains("verify") {
    return Err(AppError::Error);
  }
  let password_valid = hash::verify_bcrypt(&password, &user.password)?;
  if !password_valid {
    return Err(AppError::Error);
  }
  if let Some(secret) = user.two_factor_auth.clone() {
    if secret.len() == 32 {
      let mut totp = TOTP::default();
      let raw = Secret::Encoded(secret).to_raw()?;
      totp.secret = raw.to_bytes()?;
      if !totp.check_current(&code)? {
        return Err(AppError::TwoFactorAuth);
      }
    }
  }
  let token = jwt::sign(email, &state.jwt_token, 2592000)?;
  let mail_md5 = hash::md5(&user.email);
  let avatar = if let Some(avatar) = user.avatar {
    avatar
  } else {
    get_avatar(&user.email)
  };
  let data = json!({
    "display_name": user.display_name,
    "email": user.email,
    "password": null,
    "type": user.user_type,
    "label": user.label,
    "url": user.url,
    "avatar":avatar,
    "github": user.github,
    "twitter": user.twitter,
    "facebook": user.facebook,
    "google": user.google,
    "weibo": user.weibo,
    "qq": user.qq,
    "2fa": user.two_factor_auth,
    "createdAt": user.created_at,
    "updatedAt": user.updated_at,
    "objectId": user.id,
    "mailMd5": mail_md5,
    "token": token
  });
  Ok(data)
}

pub async fn delete_token() -> ServiceResult<Value> {
  Ok(json!({}))
}

pub async fn get_login_user_info(state: &AppState, token: String) -> ServiceResult<Value> {
  let email = jwt::verify::<String>(&token, &state.jwt_token)?.claims.data;
  let user = state
    .repo
    .user()
    .get_user_by_email(&email)
    .await?
    .ok_or(AppError::UserNotFound)?;
  let mail_md5 = hash::md5(&user.email);
  let avatar = if let Some(avatar) = user.avatar {
    avatar
  } else {
    get_avatar(&user.email)
  };
  Ok(json! ({
      "display_name": user.display_name,
      "email": user.email,
      "type": user.user_type,
      "label": user.label,
      "url": user.url,
      "avatar": avatar,
      "github": user.github,
      "twitter": user.twitter,
      "facebook": user.facebook,
      "google": user.google,
      "weibo": user.weibo,
      "qq": user.qq,
      "2fa": user.two_factor_auth,
      "objectId": user.id,
      "mailMd5": mail_md5,
  }))
}

pub async fn set_user_profile(
  state: &AppState,
  token: String,
  body: SetUserProfileBody,
) -> ServiceResult<Value> {
  let SetUserProfileBody {
    display_name,
    label,
    url,
    password,
    avatar,
    two_factor_auth,
    qq,
    weibo,
    github,
  } = body;
  let email = jwt::verify::<String>(&token, &state.jwt_token)?.claims.data;
  let mut active_user = state
    .repo
    .user()
    .get_user_by_email(&email)
    .await?
    .ok_or(AppError::UserNotFound)?
    .into_active_model();
  if let Some(display_name) = display_name {
    active_user.display_name = Set(display_name);
  }
  if let Some(label) = label {
    active_user.label = Set(Some(label));
  }
  if let Some(url) = url {
    active_user.url = Set(Some(url));
  }
  if let Some(avatar) = avatar {
    active_user.avatar = Set(Some(avatar));
  }
  if let Some(password) = password {
    let hashed = hash::bcrypt(&password)?;
    active_user.password = Set(hashed);
  }
  if let Some(two_factor_auth) = two_factor_auth {
    active_user.two_factor_auth = Set(Some(two_factor_auth));
  }
  if let Some(qq) = qq {
    active_user.qq = Set(Some(qq));
  }
  if let Some(weibo) = weibo {
    active_user.weibo = Set(Some(weibo));
  }
  if let Some(github) = github {
    active_user.github = Set(Some(github));
  }
  state.repo.user().update_user(active_user).await?;
  Ok(json!({}))
}

pub async fn set_user_type(
  state: &AppState,
  token: String,
  user_id: u32,
  r#type: String,
) -> ServiceResult<Value> {
  let email = jwt::verify::<String>(&token, &state.jwt_token)?.claims.data;
  if state.repo.user().is_admin_user(&email).await? {
    let mut active_user = state
      .repo
      .user()
      .get_user_by_id(user_id)
      .await?
      .ok_or(AppError::UserNotFound)?
      .into_active_model();
    if state.repo.user().is_first_admin_user(user_id).await? {
      return Err(AppError::Forbidden);
    }
    active_user.user_type = Set(r#type);
    state.repo.user().update_user(active_user).await?;
    Ok(json!({}))
  } else {
    Err(AppError::Forbidden)
  }
}

pub async fn get_user_info_list(state: &AppState, page: u32) -> ServiceResult<Value> {
  let page_size = 10;
  let paginator = wl_users::Entity::find()
    .select_only()
    .columns(wl_users::Column::iter().filter(|col| !matches!(col, wl_users::Column::Id)))
    .column_as(wl_users::Column::Id, "objectId")
    .into_json()
    .paginate(&state.repo.db, page_size);
  let total_pages = paginator.num_pages().await?;
  let users = paginator.fetch_page((page - 1) as u64).await?;
  Ok(json!({
    "data": users,
    "page": page,
    "pageSize": page_size,
    "totalPages": total_pages,
  }))
}

pub async fn get_user_info(state: &AppState, email: Option<String>) -> ServiceResult<Value> {
  match wl_users::Entity::find()
    .filter(wl_users::Column::Email.eq(email))
    .select_only()
    .columns(wl_users::Column::iter().filter(|col| !matches!(col, wl_users::Column::Id)))
    .column_as(wl_users::Column::Id, "objectId")
    .into_json()
    .one(&state.repo.db)
    .await?
  {
    Some(data) => Ok(data),
    None => Err(AppError::Error),
  }
}

pub async fn verification(state: &AppState, email: String, token: String) -> ServiceResult<Value> {
  let user = state
    .repo
    .user()
    .get_user_by_email(&email)
    .await?
    .ok_or(AppError::UserNotFound)?;
  tracing::debug!("type: {}", user.user_type);
  let reg = Regex::new(r"^verify:(\d{4}):(\d+)$").unwrap();
  tracing::debug!("reg {}", reg);
  let captures = reg.captures(&user.user_type).unwrap();
  tracing::debug!("captures {:#?}", captures);
  if token == captures.get(1).unwrap().as_str()
    && utc_now().timestamp_millis() < captures.get(2).unwrap().as_str().parse::<i64>().unwrap()
  {
    let mut active_user = user.into_active_model();
    active_user.user_type = Set("guest".to_string());
    state.repo.user().update_user(active_user).await?;
    return Ok(json!({}));
  }
  Err(AppError::TokenExpired)
}

pub async fn set_2fa(
  state: &AppState,
  token: String,
  code: String,
  secret: String,
) -> ServiceResult<Value> {
  let user_email = jwt::verify::<String>(&token, &state.jwt_token)?.claims.data;
  let mut user = state
    .repo
    .user()
    .get_user_by_email(&user_email)
    .await?
    .ok_or(AppError::UserNotFound)?
    .into_active_model();
  user.two_factor_auth = Set(Some(secret.clone()));
  let mut totp = TOTP::default();
  let raw = Secret::Encoded(secret.clone()).to_raw().unwrap();
  totp.secret = raw.clone().to_bytes().unwrap();
  if let Ok(check) = totp.check_current(&code) {
    if check {
      state.repo.user().set_2fa(user).await?;
      Ok(json!({}))
    } else {
      Err(AppError::TwoFactorAuth)
    }
  } else {
    Err(AppError::Error)
  }
}

pub async fn get_2fa(
  state: &AppState,
  token: Option<String>,
  email: Option<String>,
) -> ServiceResult<Value> {
  if token.is_none() && email.is_some() {
    let mut enabled = false;
    let user = wl_users::Entity::find()
      .filter(wl_users::Column::Email.eq(email))
      .filter(wl_users::Column::TwoFactorAuth.is_not_null().ne(""))
      .one(&state.repo.db)
      .await?;
    if let Some(user) = user {
      enabled = user.two_factor_auth.unwrap().len() == 32;
    }
    return Ok(json!({
        "enable": enabled
    }));
  }

  let user_email = jwt::verify::<String>(&token.unwrap(), &state.jwt_token)?
    .claims
    .data;
  let user = state
    .repo
    .user()
    .get_user_by_email(&user_email)
    .await?
    .ok_or(AppError::Error)?;
  let name = format!("waline_{}", user.id);

  if let Some(secret) = user.two_factor_auth {
    if secret.len() == 32 {
      return Ok(json!({
        "otpauth_url": format!("otpauth://totp/{name}?secret={}", secret),
        "secret": secret,
      }));
    }
  }

  let raw = Secret::generate_secret();
  let totp = TOTP {
    account_name: name.clone(),
    secret: raw.to_bytes().unwrap(),
    ..Default::default()
  };
  let token = totp.generate_current().unwrap();
  Ok(json!({
    "otpauth_url": totp.get_url(),
    "secret": totp.get_secret_base32(),
    "code":  token,
  }))
}

pub async fn modify_password(
  state: &AppState,
  email: String,
  origin: &str,
  lang: &str,
) -> ServiceResult<Value> {
  let EnvConfig {
    smtp_service,
    smtp_host,
    ..
  } = EnvConfig::load_env()?;

  if smtp_service.is_none() || smtp_host.is_none() {
    return Err(AppError::Error);
  }

  match state.repo.user().get_user_by_email(&email).await? {
    Some(user) => {
      let token = jwt::sign(user.email.clone(), &state.jwt_token, 300)?;
      let url = format!("{origin}/ui/profile?token={token}");
      send_email_notification(Notification {
        notify_type: NotifyType::ResetPassword,
        sender_name: "".to_owned(),
        sender_email: user.email,
        comment_id: 0,
        comment: "".to_owned(),
        url,
        lang: Some(lang),
      });
    }
    _ => {
      return Err(AppError::Error);
    }
  }

  Ok(json!(()))
}
