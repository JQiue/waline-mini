use actix_web::{
  HttpRequest, HttpResponse, get,
  http::{self, header::ContentType},
  web::{Data, Query},
};
use helpers::jwt;

use crate::{
  app::AppState,
  components::ui::{model::*, service},
  error::AppError,
  helpers::header::extract_token,
};

#[get("/profile")]
pub async fn ui_profile_page(
  state: Data<AppState>,
  query: Query<UIProfilePageQuery>,
) -> HttpResponse {
  if let Some(token) = query.0.token {
    if jwt::verify::<String>(&token, &state.jwt_token).is_ok() {
      HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(service::admin_page().await)
    } else {
      HttpResponse::Found()
        .append_header((http::header::LOCATION, "/ui/login".to_string()))
        .finish()
    }
  } else {
    HttpResponse::Ok()
      .content_type(ContentType::html())
      .body(service::admin_page().await)
  }
}

#[get("/login")]
pub async fn ui_login_page(
  req: HttpRequest,
  state: Data<AppState>,
  query: Query<UiLoginPageQeury>,
) -> Result<HttpResponse, AppError> {
  if let Ok(token) = extract_token(&req) {
    if let Ok(_) = jwt::verify::<String>(&token, &state.jwt_token)
      && let Some(redirect) = query.0.redirect
    {
      return Ok(
        HttpResponse::Found()
          .append_header((http::header::LOCATION, redirect))
          .finish(),
      );
    }
  }

  Ok(
    HttpResponse::Ok()
      .content_type(ContentType::html())
      .body(service::admin_page().await),
  )
}

#[get("/migration")]
pub async fn ui_migration_page() -> HttpResponse {
  HttpResponse::Ok()
    .content_type(ContentType::html())
    .body(service::admin_page().await)
}

#[get("/user")]
pub async fn ui_user_page() -> HttpResponse {
  HttpResponse::Ok()
    .content_type(ContentType::html())
    .body(service::admin_page().await)
}

#[get("/forgot")]
pub async fn ui_forgot_page() -> HttpResponse {
  HttpResponse::Ok()
    .content_type(ContentType::html())
    .body(service::admin_page().await)
}

pub async fn ui_page() -> HttpResponse {
  HttpResponse::Ok()
    .content_type(ContentType::html())
    .body(service::admin_page().await)
}
