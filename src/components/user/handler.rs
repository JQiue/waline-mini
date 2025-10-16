use actix_web::{
  HttpRequest, HttpResponse, delete, get,
  http::{self},
  post, put,
  web::{Data, Json, Path, Query},
};
use ammonia::url;
use helpers::{hash, jwt, uuid::Alphabet};
use sea_orm::{ActiveValue::Set, IntoActiveModel};
use serde_json::json;

use crate::{
  app::AppState,
  components::user::{model::*, service},
  config::EnvConfig,
  entities::wl_users,
  error::AppError,
  helpers::header::{extract_host, extract_origin, extract_token, get_server_url},
  prelude::*,
  repository::user::UserQueryBySocial,
};

#[post("/user")]
pub async fn user_register(
  req: HttpRequest,
  state: Data<AppState>,
  query: Query<UserRegisterQuery>,
  body: Json<UserRegisterBody>,
) -> Result<HttpResponse, AppError> {
  service::user_register(&state, body.0, extract_host(&req), &query.0.lang)
    .await
    .into_http_response(Some(&query.0.lang))
}

#[post("/token")]
pub async fn user_login(
  state: Data<AppState>,
  body: Json<UserLoginBody>,
) -> Result<HttpResponse, AppError> {
  service::user_login(&state, body.0)
    .await
    .into_http_response(None)
}

#[delete("/token")]
pub async fn user_logout() -> Result<HttpResponse, AppError> {
  service::delete_token().await.into_http_response(None)
}

#[get("/token")]
async fn get_login_user_info(
  req: HttpRequest,
  state: Data<AppState>,
) -> Result<HttpResponse, AppError> {
  service::get_login_user_info(&state, extract_token(&req)?)
    .await
    .into_http_response(None)
}

#[put("/user")]
pub async fn set_user_profile(
  req: HttpRequest,
  state: Data<AppState>,
  body: Json<SetUserProfileBody>,
) -> Result<HttpResponse, AppError> {
  service::set_user_profile(&state, extract_token(&req)?, body.0)
    .await
    .into_http_response(None)
}

// WARNING
#[put("/user/{user_id}")]
pub async fn set_user_type(
  req: HttpRequest,
  state: Data<AppState>,
  path: Path<u32>,
  body: Json<SetUserTypeBody>,
) -> Result<HttpResponse, AppError> {
  let user_id = path.into_inner();
  let Json(SetUserTypeBody { r#type }) = body;
  service::set_user_type(&state, extract_token(&req)?, user_id, r#type)
    .await
    .into_http_response(None)
}

#[get("/user")]
pub async fn get_user_info(
  state: Data<AppState>,
  query: Query<GetUserQuery>,
) -> Result<HttpResponse, AppError> {
  let Query(GetUserQuery { email, lang, page }) = query;
  if let Some(page) = page {
    service::get_user_info_list(&state, page)
      .await
      .into_http_response(Some(&lang))
  } else {
    service::get_user_info(&state, email)
      .await
      .into_http_response(None)
  }
}

#[get("/verification")]
pub async fn verification(
  state: Data<AppState>,
  query: Query<VerificationQuery>,
) -> Result<HttpResponse, AppError> {
  let Query(VerificationQuery { email, token }) = query;
  let r = service::verification(&state, email, token)
    .await
    .into_http_response(None);

  if r.is_ok() {
    Ok(
      HttpResponse::Found()
        .append_header((http::header::LOCATION, "/ui/login"))
        .finish(),
    )
  } else {
    r
  }
}

#[post("/token/2fa")]
pub async fn set_2fa(
  req: HttpRequest,
  state: Data<AppState>,
  body: Json<Set2faBody>,
) -> Result<HttpResponse, AppError> {
  let Json(Set2faBody { code, secret }) = body;
  let token = extract_token(&req)?;
  service::set_2fa(&state, token, code, secret)
    .await
    .into_http_response(None)
}

#[get("/token/2fa")]
pub async fn get_2fa(
  req: HttpRequest,
  state: Data<AppState>,
  query: Query<Get2faQuery>,
) -> Result<HttpResponse, AppError> {
  let Query(Get2faQuery { lang, email }) = query;
  let token = extract_token(&req).ok();
  service::get_2fa(&state, token, email)
    .await
    .into_http_response(Some(&lang))
}

#[put("/user/password")]
pub async fn modify_password(
  req: HttpRequest,
  state: Data<AppState>,
  query: Query<UserPasswordQuery>,
  body: Json<UserPasswordBody>,
) -> Result<HttpResponse, AppError> {
  let Query(UserPasswordQuery { lang }) = query;
  let Json(UserPasswordBody { email }) = body;
  let origin = extract_origin(&req);
  service::modify_password(&state, email, &origin, &lang)
    .await
    .into_http_response(Some(&lang))
}

#[get("/oauth")]
pub async fn oauth(req: HttpRequest, query: Query<OAuthQuery>) -> Result<HttpResponse, AppError> {
  let Query(OAuthQuery {
    r#type,
    redirect: _,
    state,
  }) = query.clone();

  let EnvConfig { oauth_url, .. } = EnvConfig::load_env()?;

  let server_url = get_server_url(&req)?;
  let mut redirect_url_params = url::form_urlencoded::Serializer::new(String::new());
  let mut oauth_params = url::form_urlencoded::Serializer::new(String::new());
  redirect_url_params.append_pair("type", &r#type);
  if let Some(state) = state
    && state.len() != 0
  {
    oauth_params.append_pair("state", &state);
  };

  let redirect_url = format!(
    "{server_url}/api/oauth/callback?{}",
    redirect_url_params.finish()
  );
  println!("redirect_url={}", redirect_url);
  oauth_params.append_pair("redirect", &redirect_url);
  let oauth_url = format!("{oauth_url}/{}?{}", r#type, oauth_params.finish());
  return Ok(
    HttpResponse::Found()
      .append_header((http::header::LOCATION, oauth_url))
      .finish(),
  );
}

#[get("/oauth/callback")]
pub async fn oauth_callback(
  state: Data<AppState>,
  query: Query<OAuthCallbackQuery>,
) -> Result<HttpResponse, AppError> {
  let OAuthCallbackQuery {
    oauth_verifier,
    oauth_token,
    code,
    r#type,
    state: token,
  } = query.0;

  let EnvConfig { oauth_url, .. } = EnvConfig::load_env()?;
  // 已经拥有了 code 或者 X 平台必须的：oauth_token 和 oauth_verifier
  let params = json!({
    "code": code,
    "oauth_token": oauth_token,
    "oauth_verifier": oauth_verifier,
  });

  // 获取第三方用户信息
  let url = format!("{oauth_url}/{}?code={}", r#type, code);
  let resp = reqwest::get(url).await.unwrap();
  let mut user_by_oauth = resp.json::<OauthUserInfo>().await.unwrap();

  // 通过 user.id 和 r#type 去数据库查找用户
  let social_type = match r#type.as_str() {
    "qq" => UserQueryBySocial::QQ,
    "weibo" => UserQueryBySocial::Weibo,
    "github" => UserQueryBySocial::Github,
    _ => return Err(AppError::Error),
  };
  let user_by_social = state
    .repo
    .user()
    .get_user_by_social(social_type, &user_by_oauth.id)
    .await?;

  // 如果通过社交找到用户，则是签发 token 进行登录
  if let Some(user) = user_by_social {
    let token = jwt::sign(user.email, &state.jwt_token, 2592000)?;
    let redirect_url = format!("/ui/profile?token={token}");
    return Ok(
      HttpResponse::Found()
        .append_header((http::header::LOCATION, redirect_url))
        .finish(),
    );
  }

  // 否则，拼接用户第三方 id 查找用户 id
  if user_by_oauth.email.is_none() {
    user_by_oauth.email = Some(format!("{}@mail.{}", user_by_oauth.id, r#type.clone()));
  }

  // 如果携带了 state，意味着是个关联操作
  if let Some(token) = token
    && token != ""
  {
    let email = jwt::verify::<String>(&token, &state.jwt_token)?.claims.data;
    let user_by_current: Option<wl_users::Model> =
      state.repo.user().get_user_by_email(&email).await?;
    if let Some(user) = user_by_current {
      let mut active_user = user.clone().into_active_model();
      match r#type.as_str() {
        "qq" => active_user.qq = Set(Some(user_by_oauth.id)),
        "weibo" => active_user.weibo = Set(Some(user_by_oauth.id)),
        "github" => active_user.github = Set(Some(user_by_oauth.id)),
        _ => return Err(AppError::Error),
      };
      if user.avatar.is_none() && user_by_oauth.avatar.is_some() {
        active_user.avatar = Set(user_by_oauth.avatar);
      }
      state.repo.user().update_user(active_user).await?;
      println!("关联当前用户");
      return Ok(
        HttpResponse::Found()
          .append_header((http::header::LOCATION, "/ui/profile"))
          .finish(),
      );
    }
  }

  let user_by_email = state
    .repo
    .user()
    .get_user_by_email(&user_by_oauth.email.clone().unwrap())
    .await?;

  // 尝试通过拼接 id 查找用户，如果有则更新，没有则创建
  if let Some(user) = user_by_email {
    let mut active_user = user.clone().into_active_model();
    match r#type.as_str() {
      "qq" => active_user.qq = Set(Some(user_by_oauth.id)),
      "weibo" => active_user.weibo = Set(Some(user_by_oauth.id)),
      "github" => active_user.github = Set(Some(user_by_oauth.id)),
      _ => return Err(AppError::Error),
    };
    if user.avatar.is_none() && user_by_oauth.avatar.is_some() {
      active_user.avatar = Set(user_by_oauth.avatar);
    }
    state.repo.user().update_user(active_user).await?;
  } else {
    let user_type = if state.repo.user().is_first_user().await? {
      "administrator"
    } else {
      "guest"
    };
    let hashed = hash::bcrypt_custom(
      &helpers::uuid::nanoid(&Alphabet::DEFAULT, 8),
      8,
      helpers::hash::Version::TwoA,
    )?;
    let mut active_user: wl_users::ActiveModel = wl_users::ActiveModel {
      display_name: Set(user_by_oauth.name),
      email: Set(user_by_oauth.email.clone().unwrap()),
      avatar: Set(user_by_oauth.avatar),
      password: Set(hashed),
      user_type: Set(user_type.to_string()),
      url: Set(Some("".to_string())),
      ..Default::default()
    };
    match r#type.as_str() {
      "qq" => active_user.qq = Set(Some(user_by_oauth.id)),
      "weibo" => active_user.weibo = Set(Some(user_by_oauth.id)),
      "github" => active_user.github = Set(Some(user_by_oauth.id)),
      _ => return Err(AppError::Error),
    };
    println!("创建用户");
    state.repo.user().create_user(active_user).await?;
  }

  let token = jwt::sign(user_by_oauth.email, &state.jwt_token, 2592000)?;
  let redirect_url = format!("/ui/profile?token={token}");

  return Ok(
    HttpResponse::Found()
      .append_header((http::header::LOCATION, redirect_url))
      .finish(),
  );
}
