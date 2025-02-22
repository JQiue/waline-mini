use helpers::{
  hash, jwt,
  time::utc_now,
  uuid::{self, Alphabet},
};
use regex::Regex;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, Iterable, PaginatorTrait,
  QueryFilter, QuerySelect, Set,
};
use serde_json::{json, Value};
use totp_rs::{Secret, TOTP};

use crate::{
  app::AppState,
  components::user::model::{has_user, is_first_user, UserQueryBy},
  config::EnvConfig,
  entities::*,
  error::AppError,
  helpers::{
    avatar::get_avatar,
    email::{send_email_notification, CommentNotification, NotifyType},
  },
  response::Code,
};

use super::model::{get_user, is_admin_user, is_first_admin_user};

pub async fn user_register(
  state: &AppState,
  display_name: String,
  email: String,
  password: String,
  url: String,
  host_header: String,
  lang: &str,
) -> Result<Value, Code> {
  let mut data = json!({
    "verify": true
  });
  let hashed: String =
    hash::bcrypt_custom(&password, 8, helpers::hash::Version::TwoA).map_err(|_| Code::Error)?;
  let EnvConfig { site_name, .. } = EnvConfig::load_env().unwrap();
  if has_user(UserQueryBy::Email(email.clone()), &state.conn).await? {
    let user = get_user(UserQueryBy::Email(email.clone()), &state.conn).await?;
    if user.user_type != "administrator" || user.user_type != "guest" {
      let mut active_user = user.into_active_model();
      active_user.display_name = Set(display_name);
      active_user.url = Set(Some(url));
      active_user.password = Set(hashed);
      let token = uuid::uuid(&Alphabet::NUMBERS, 4);
      active_user.user_type = Set(format!(
        "verify:{}:{}",
        token,
        utc_now().timestamp_millis() + 60 * 60 * 1000
      ));
      let url = format!(
        "http://{}/api/verification?token={}&email={}",
        host_header, token, email
      );
      send_email_notification(CommentNotification {
        sender_name: site_name,
        sender_email: email,
        comment_id: 0,
        comment: "".to_string(),
        url,
        notify_type: NotifyType::Notify,
        lang: Some(lang),
      });
      return match active_user
        .update(&state.conn)
        .await
        .map_err(AppError::from)
      {
        Ok(_) => Ok(data),
        Err(err) => Err(err.into()),
      };
    }
    return Err(Code::UserRegistered);
  }
  let mut user = wl_users::ActiveModel {
    display_name: Set(display_name),
    email: Set(email.clone()),
    url: Set(Some(url)),
    password: Set(hashed),
    ..Default::default()
  };
  if is_first_user(&state.conn).await? {
    user.user_type = Set("administrator".to_string());
    data = json!({});
  } else {
    let token = uuid::uuid(&Alphabet::NUMBERS, 4);
    user.user_type = Set(format!(
      "verify:{}:{}",
      token,
      utc_now().timestamp_millis() + 60 * 60 * 1000
    ));
    let url = format!(
      "http://{}/api/verification?token={}&email={}",
      host_header, token, email
    );
    send_email_notification(CommentNotification {
      sender_name: site_name,
      sender_email: email,
      comment_id: 0,
      comment: "".to_string(),
      url,
      notify_type: NotifyType::Notify,
      lang: Some(lang),
    });
  }
  match user.insert(&state.conn).await.map_err(AppError::from) {
    Ok(_) => Ok(data),
    Err(err) => Err(err.into()),
  }
}

pub async fn user_login(
  state: &AppState,
  code: String,
  email: String,
  password: String,
) -> Result<Value, Code> {
  let user = get_user(UserQueryBy::Email(email.clone()), &state.conn).await?;
  if user.user_type.contains("verify") {
    return Err(Code::Error);
  }
  let result = hash::verify_bcrypt(&password, &user.password).map_err(AppError::from)?;
  if !result {
    return Err(Code::Error);
  }
  if let Some(secret) = user.two_factor_auth.clone() {
    if secret.len() == 32 {
      let mut totp = TOTP::default();
      let raw = Secret::Encoded(secret).to_raw().unwrap();
      totp.secret = raw.to_bytes().unwrap();
      if !totp.check_current(&code).unwrap() {
        return Err(Code::TwoFactorAuth);
      }
    }
  }
  let token = jwt::sign(user.email.clone(), &state.jwt_token, 2592000).map_err(AppError::from)?;
  let mail_md5 = helpers::hash::md5(user.email.as_bytes());
  let data = json!({
    "display_name": user.display_name,
    "email": user.email,
    "password": null,
    "type": user.user_type,
    "label": user.label,
    "url": user.url,
    "avatar": get_avatar(&user.email),
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

pub async fn get_login_user_info(state: &AppState, token: String) -> Result<Value, Code> {
  let email = helpers::jwt::verify::<String>(&token, &state.jwt_token)
    .map_err(AppError::from)?
    .claims
    .data;
  let user = get_user(UserQueryBy::Email(email), &state.conn).await?;
  let mail_md5 = helpers::hash::md5(user.email.as_bytes());
  Ok(json! ({
      "display_name": user.display_name,
      "email": user.email,
      "type": user.user_type,
      "label": user.label,
      "url": user.url,
      "avatar": get_avatar(&user.email),
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
  display_name: Option<String>,
  label: Option<String>,
  url: Option<String>,
  password: Option<String>,
  avatar: Option<String>,
  two_factor_auth: Option<String>,
) -> Result<bool, Code> {
  let email = jwt::verify::<String>(&token, &state.jwt_token)
    .map_err(AppError::from)?
    .claims
    .data;
  let mut active_user = get_user(UserQueryBy::Email(email), &state.conn)
    .await?
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
    let hashed = helpers::hash::bcrypt(&password).map_err(|_| Code::Error)?;
    active_user.password = Set(hashed);
  }
  if let Some(two_factor_auth) = two_factor_auth {
    active_user.two_factor_auth = Set(Some(two_factor_auth));
  }
  let res = active_user.update(&state.conn).await;
  Ok(res.is_ok())
}

pub async fn set_user_type(
  state: &AppState,
  token: String,
  user_id: u32,
  r#type: String,
) -> Result<bool, Code> {
  let email = jwt::verify::<String>(&token, &state.jwt_token)
    .map_err(|_| Code::Unauthorized)?
    .claims
    .data;
  if is_admin_user(&email, &state.conn).await? {
    let mut active_user = get_user(UserQueryBy::Id(user_id), &state.conn)
      .await?
      .into_active_model();
    if is_first_admin_user(user_id, &state.conn).await? {
      return Err(Code::Forbidden);
    }
    active_user.user_type = Set(r#type);
    active_user
      .update(&state.conn)
      .await
      .map_err(|_| AppError::Database)?;
    Ok(true)
  } else {
    Err(Code::Forbidden)
  }
}

pub async fn get_user_info_list(state: &AppState, page: u32) -> Result<Value, Code> {
  let page_size = 10;
  let paginator = wl_users::Entity::find()
    .select_only()
    .columns(wl_users::Column::iter().filter(|col| !matches!(col, wl_users::Column::Id)))
    .column_as(wl_users::Column::Id, "objectId")
    .into_json()
    .paginate(&state.conn, page_size);
  let total_pages = paginator.num_pages().await.map_err(AppError::from)?;
  let users = paginator
    .fetch_page((page - 1) as u64)
    .await
    .map_err(AppError::from)?;
  Ok(json!({
    "data": users,
    "page": page,
    "pageSize": page_size,
    "totalPages": total_pages,
  }))
}

pub async fn get_user_info(state: &AppState, email: Option<String>) -> Result<Value, Code> {
  match wl_users::Entity::find()
    .filter(wl_users::Column::Email.eq(email))
    .select_only()
    .columns(wl_users::Column::iter().filter(|col| !matches!(col, wl_users::Column::Id)))
    .column_as(wl_users::Column::Id, "objectId")
    .into_json()
    .one(&state.conn)
    .await
    .map_err(AppError::from)?
  {
    Some(data) => Ok(data),
    None => Err(Code::Error),
  }
}

pub async fn verification(state: &AppState, email: String, token: String) -> Result<bool, Code> {
  let user = get_user(UserQueryBy::Email(email), &state.conn)
    .await
    .map_err(AppError::from)?;
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
    active_user
      .update(&state.conn)
      .await
      .map_err(AppError::from)?;
    return Ok(true);
  }
  Err(Code::TokenExpired)
}

pub async fn set_2fa(
  state: &AppState,
  token: String,
  code: String,
  secret: String,
) -> Result<Value, Code> {
  let user_email = jwt::verify::<String>(&token, &state.jwt_token)
    .map_err(AppError::from)?
    .claims
    .data;
  let mut user = state
    .repo
    .user()
    .get_user_by_email(&user_email)
    .await
    .map_err(AppError::from)?
    .ok_or(AppError::UserNotFound)?
    .into_active_model();
  user.two_factor_auth = Set(Some(secret.clone()));
  let mut totp = TOTP::default();
  let raw = Secret::Encoded(secret.clone()).to_raw().unwrap();
  totp.secret = raw.clone().to_bytes().unwrap();
  if let Ok(check) = totp.check_current(&code) {
    if check {
      state
        .repo
        .user()
        .set_2fa(user)
        .await
        .map_err(AppError::from)?;
      return Ok(json!({}));
    } else {
      return Err(Code::TwoFactorAuth);
    }
  } else {
    return Err(Code::Error);
  }
}

pub async fn get_2fa(
  state: &AppState,
  token: Option<String>,
  email: Option<String>,
) -> Result<Value, Code> {
  if token.is_none() && email.is_some() {
    let user = wl_users::Entity::find()
      .filter(wl_users::Column::Email.eq(email))
      .filter(wl_users::Column::TwoFactorAuth.is_not_null().ne(""))
      .one(&state.conn)
      .await
      .map_err(AppError::from)?;
    let mut enabled = false;
    if let Some(user) = user {
      enabled = user.two_factor_auth.unwrap().len() == 32;
    }
    return Ok(json!({
        "enable": enabled
    }));
  }
  let user_email = jwt::verify::<String>(&token.unwrap(), &state.jwt_token)
    .map_err(AppError::from)?
    .claims
    .data;
  let user = state
    .repo
    .user()
    .get_user_by_email(&user_email)
    .await
    .map_err(AppError::from)?
    .ok_or(Code::Error)?;
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
  let mut totp = TOTP::default();
  totp.account_name = name.clone();
  totp.secret = raw.to_bytes().unwrap();
  let token = totp.generate_current().unwrap();
  return Ok(json!({
    "otpauth_url": totp.get_url(),
    "secret": totp.get_secret_base32(),
    "code":  token,
  }));
}
