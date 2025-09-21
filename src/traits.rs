use actix_web::HttpResponse;
use serde::Serialize;

use crate::prelude::*;

pub trait LoggingResultErr<T, E> {
  fn log_err(self) -> Result<T, E>;
}

impl<T, E: std::fmt::Debug> LoggingResultErr<T, E> for Result<T, E> {
  fn log_err(self) -> Result<T, E> {
    if let Err(ref e) = self {
      tracing::error!("{:?}", e);
    }
    self
  }
}

pub trait IntoHttpResponse<T> {
  fn into_http_response(self, lang: Option<&str>) -> Result<HttpResponse, AppError>;
}

impl<T> IntoHttpResponse<T> for ServiceResult<T>
where
  T: Serialize,
{
  fn into_http_response(self, lang: Option<&str>) -> Result<HttpResponse, AppError> {
    match self {
      Ok(data) => Response::<T>::new_success(Some(data)),
      Err(err) => Response::<T>::new_error(err, lang),
    }
  }
}
