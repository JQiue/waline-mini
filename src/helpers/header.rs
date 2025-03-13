use actix_web::HttpRequest;

use crate::error::AppError;

pub fn extract_token(req: &HttpRequest) -> Result<String, AppError> {
  let auth_header = req
    .headers()
    .get("Authorization")
    .ok_or(AppError::Unauthorized)?
    .to_str()?;
  if !auth_header.starts_with("Bearer ") {
    return Err(AppError::Unauthorized);
  }
  Ok(auth_header[7..].to_string()) // Skip "Bearer " prefix
}

pub fn extract_ip(req: &HttpRequest) -> String {
  if let Some(h) = req.headers().get("X-Forwarded-For") {
    let s = h.to_str().unwrap_or("0.0.0.0").to_string();
    s
  } else if let Some(h) = req.headers().get("X-Real-IP") {
    let s = h.to_str().ok().unwrap_or("0.0.0.0").to_string();
    s
  } else {
    req
      .peer_addr()
      .map(|s| s.ip().to_string())
      .unwrap_or_default()
  }
}

pub fn extract_referer(req: &HttpRequest) -> Option<String> {
  req
    .headers()
    .get("Referer")
    .and_then(|h| h.to_str().ok())
    .map(|s| s.to_string())
}

pub fn extract_origin(req: &HttpRequest) -> String {
  req
    .headers()
    .get("Origin")
    .and_then(|h| h.to_str().ok())
    .unwrap_or_default()
    .to_string()
}
