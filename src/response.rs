use std::fmt::Display;

use actix_web::HttpResponse;
use serde::Serialize;

use crate::{error::AppError, locales::get_translation};

#[derive(Debug, Serialize)]
pub struct Response<T> {
  pub errno: i32,
  pub errmsg: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub data: Option<T>,
}

impl<T> Response<T> {
  pub fn success(data: Option<T>) -> Self {
    Self {
      data,
      errno: 0,
      errmsg: "".to_string(),
    }
  }

  pub fn error(error: AppError, lang: Option<&str>) -> Self {
    Self {
      data: None,
      errno: error.code(),
      errmsg: get_translation(lang.unwrap_or("en"), &error.message()),
    }
  }

  pub fn new_success(data: Option<T>) -> Result<HttpResponse, AppError>
  where
    T: Serialize,
  {
    Ok(HttpResponse::Ok().json(Response {
      data,
      errno: 0,
      errmsg: "".to_string(),
    }))
  }

  pub fn new_error(error: AppError, lang: Option<&str>) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(Response::<()> {
      data: None,
      errno: error.code(),
      errmsg: get_translation(lang.unwrap_or("en"), &error.message()),
    }))
  }
}

impl<T> Display for Response<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      r#"{{ "errno": {}, "errmsg": "{}" }}"#,
      self.errno, self.errmsg
    )
  }
}
