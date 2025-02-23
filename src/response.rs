use std::fmt::Display;

use actix_web::HttpResponse;
use serde::Serialize;

use crate::{error::AppError, locales::get_translation};

/// Response code enumeration
// #[derive(Debug, Clone, Copy, Serialize)]
// pub enum Code {
//   Success,
//   Error,
//   UserRegistered,
//   DuplicateContent,
//   Unauthorized,
//   FrequencyLimited,
//   TokenExpired,
//   Forbidden,
//   TwoFactorAuth,
// }

// impl Code {
//   pub fn message(&self, lang: &str) -> String {
//     match self {
//       Code::Success => "".to_owned(),
//       Code::UserRegistered => get_translation(lang, "USER_REGISTERED"),
//       Code::Error => "".to_owned(),
//       Code::DuplicateContent => get_translation(lang, "Duplicate Content"),
//       Code::Unauthorized => get_translation(lang, "Unauthorized"),
//       Code::FrequencyLimited => get_translation(lang, "Comment too fast"),
//       Code::TokenExpired => get_translation(lang, "TOKEN_EXPIRED"),
//       Code::Forbidden => get_translation(lang, "FORBIDDEN"),
//       Code::TwoFactorAuth => get_translation(lang, "TWO_FACTOR_AUTH_ERROR_DETAIL"),
//     }
//   }
// }

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
