use actix_web::{HttpResponse, ResponseError, http::StatusCode};

#[derive(Debug)]
pub enum AppError {
  Error,
  Database,
  UserNotFound,
  Akismet,
  UserRegistered,
  DuplicateContent,
  Unauthorized,
  FrequencyLimited,
  TokenExpired,
  Forbidden,
  TwoFactorAuth,
}

impl AppError {
  pub fn code(&self) -> i32 {
    match self {
      Self::Error => 1000,
      Self::Database => 1000,
      Self::UserNotFound => 1000,
      Self::Akismet => 1000,
      Self::UserRegistered => 1000,
      Self::DuplicateContent => 1000,
      Self::Unauthorized => 401,
      Self::FrequencyLimited => 1000,
      Self::TokenExpired => 1000,
      Self::Forbidden => 403,
      Self::TwoFactorAuth => 1000,
    }
  }
  pub fn message(&self) -> String {
    match self {
      AppError::Error => "".to_string(),
      AppError::Database => "".to_string(),
      AppError::UserNotFound => "".to_string(),
      AppError::Akismet => "".to_string(),
      AppError::UserRegistered => "USER_REGISTERED".to_string(),
      AppError::DuplicateContent => "Duplicate Content".to_string(),
      AppError::Unauthorized => "Unauthorized".to_string(),
      AppError::FrequencyLimited => "Comment too fast".to_string(),
      AppError::TokenExpired => "TOKEN_EXPIRED".to_string(),
      AppError::Forbidden => "FORBIDDEN".to_string(),
      AppError::TwoFactorAuth => "TWO_FACTOR_AUTH_ERROR_DETAIL".to_string(),
    }
  }
}

impl From<sea_orm::DbErr> for AppError {
  fn from(err: sea_orm::DbErr) -> Self {
    tracing::error!("{:#?}", err);
    AppError::Database
  }
}

impl From<std::io::Error> for AppError {
  fn from(err: std::io::Error) -> Self {
    tracing::error!("{:#?}", err);
    AppError::Error
  }
}

impl From<envy::Error> for AppError {
  fn from(err: envy::Error) -> Self {
    tracing::error!("{:#?}", err);
    AppError::Error
  }
}

impl From<helpers::jwt::Error> for AppError {
  fn from(err: helpers::jwt::Error) -> Self {
    tracing::error!("{:#?}", err);
    AppError::Unauthorized
  }
}

impl From<helpers::hash::BcryptError> for AppError {
  fn from(err: helpers::hash::BcryptError) -> Self {
    tracing::error!("{:#?}", err);
    AppError::Error
  }
}

impl From<actix_web::http::header::ToStrError> for AppError {
  fn from(err: actix_web::http::header::ToStrError) -> Self {
    tracing::error!("{:#?}", err);
    AppError::Error
  }
}

impl From<instant_akismet::Error> for AppError {
  fn from(err: instant_akismet::Error) -> Self {
    tracing::error!("{:#?}", err);
    AppError::Akismet
  }
}

impl From<totp_rs::SecretParseError> for AppError {
  fn from(err: totp_rs::SecretParseError) -> Self {
    tracing::error!("{:#?}", err);
    AppError::TwoFactorAuth
  }
}

impl From<std::time::SystemTimeError> for AppError {
  fn from(err: std::time::SystemTimeError) -> Self {
    tracing::error!("{:#?}", err);
    AppError::TwoFactorAuth
  }
}

impl ResponseError for AppError {
  fn status_code(&self) -> StatusCode {
    StatusCode::OK
    // match self {
    //   AppError::Error => StatusCode::OK,
    //   AppError::Database => StatusCode::OK,
    //   AppError::UserNotFound => StatusCode::OK,
    //   AppError::Authorization => StatusCode::OK,
    //   AppError::Akismet => StatusCode::OK,
    //   AppError::UserRegistered => StatusCode::OK,
    //   AppError::DuplicateContent => StatusCode::OK,
    //   AppError::Unauthorized => StatusCode::OK,
    //   AppError::FrequencyLimited => StatusCode::OK,
    //   AppError::TokenExpired => StatusCode::OK,
    //   AppError::Forbidden => StatusCode::OK,
    //   AppError::TwoFactorAuth => StatusCode::OK,
    // }
  }
  fn error_response(&self) -> HttpResponse {
    HttpResponse::build(self.status_code()).body(self.to_string())
  }
}

impl std::fmt::Display for AppError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.message())
    // match self {
    //   AppError::Error => write!(f, "Internal server error"),
    //   AppError::Database => write!(f, "Database error"),
    //   AppError::UserNotFound => write!(f, "User not found"),
    //   AppError::Authorization => write!(f, "Authorization error"),
    //   AppError::Akismet => write!(f, "Akismet error"),
    //   AppError::UserRegistered => write!(f, "User already registered"),
    //   AppError::DuplicateContent => write!(f, "Duplicate content"),
    //   AppError::Unauthorized => todo!(),
    //   AppError::FrequencyLimited => todo!(),
    //   AppError::TokenExpired => todo!(),
    //   AppError::Forbidden => todo!(),
    //   AppError::TwoFactorAuth => todo!(),
    // }
  }
}
// AppError::DatabaseError => write!(f, "Database error"),
// AppError::Error => write!(f, "Internal server error"),
// AppError::UserNotFound => write!(f, "User not found"),
// AppError::AuthorizationError => write!(f, "Authorization error"),
