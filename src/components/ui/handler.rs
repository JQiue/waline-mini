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
  helpers::header::{extract_token, get_server_url},
};

#[get("/profile")]
pub async fn ui_profile_page(
  req: HttpRequest,
  state: Data<AppState>,
  query: Query<UIProfilePageQuery>,
) -> Result<HttpResponse, AppError> {
  let server_url = get_server_url(&req)?;
  if let Some(token) = query.0.token {
    if jwt::verify::<String>(&token, &state.jwt_token).is_ok() {
      Ok(
        HttpResponse::Ok()
          .content_type(ContentType::html())
          .body(service::admin_page(&state.site_url, &state.site_name, &server_url).await),
      )
    } else {
      Ok(
        HttpResponse::Found()
          .append_header((http::header::LOCATION, "/ui/login".to_string()))
          .finish(),
      )
    }
  } else {
    Ok(
      HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(service::admin_page(&state.site_url, &state.site_name, &server_url).await),
    )
  }
}

#[get("/login")]
pub async fn ui_login_page(
  req: HttpRequest,
  state: Data<AppState>,
  query: Query<UiLoginPageQeury>,
) -> Result<HttpResponse, AppError> {
  let server_url = get_server_url(&req)?;
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
      .body(service::admin_page(&state.site_url, &state.site_name, &server_url).await),
  )
}

#[get("/migration")]
pub async fn ui_migration_page(
  req: HttpRequest,
  state: Data<AppState>,
) -> Result<HttpResponse, AppError> {
  let server_url = get_server_url(&req)?;
  Ok(
    HttpResponse::Ok()
      .content_type(ContentType::html())
      .body(service::admin_page(&state.site_url, &state.site_name, &server_url).await),
  )
}

#[get("/user")]
pub async fn ui_user_page(
  req: HttpRequest,
  state: Data<AppState>,
) -> Result<HttpResponse, AppError> {
  let server_url = get_server_url(&req)?;
  Ok(
    HttpResponse::Ok()
      .content_type(ContentType::html())
      .body(service::admin_page(&state.site_url, &state.site_name, &server_url).await),
  )
}

#[get("/forgot")]
pub async fn ui_forgot_page(
  req: HttpRequest,
  state: Data<AppState>,
) -> Result<HttpResponse, AppError> {
  let server_url = get_server_url(&req)?;
  Ok(
    HttpResponse::Ok()
      .content_type(ContentType::html())
      .body(service::admin_page(&state.site_url, &state.site_name, &server_url).await),
  )
}

pub async fn ui_page(req: HttpRequest, state: Data<AppState>) -> Result<HttpResponse, AppError> {
  let server_url = get_server_url(&req)?;
  Ok(
    HttpResponse::Ok()
      .content_type(ContentType::html())
      .body(service::admin_page(&state.site_url, &state.site_name, &server_url).await),
  )
}
