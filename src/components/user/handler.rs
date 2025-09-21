use actix_web::{
  HttpRequest, HttpResponse, delete, get,
  http::{self},
  post, put,
  web::{Data, Json, Path, Query},
};

use crate::{
  app::AppState,
  components::user::{model::*, service},
  error::AppError,
  helpers::header::{extract_host, extract_origin, extract_token},
  prelude::*,
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
