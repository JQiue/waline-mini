use std::future::{Ready, ready};

use actix_web::{
  Error, HttpResponse,
  body::EitherBody,
  dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures_util::{FutureExt as _, TryFutureExt as _, future::LocalBoxFuture};

use crate::helpers::header::{extract_origin, extract_referer};

#[derive(Clone, Debug)]
pub struct SecureDomians {
  secure_domians: Vec<String>,
}

impl SecureDomians {
  pub fn new(secure_domians: Vec<String>) -> Self {
    Self { secure_domians }
  }
}

impl<S, B> Transform<S, ServiceRequest> for SecureDomians
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
  S::Future: 'static,
  B: 'static,
{
  type Response = ServiceResponse<EitherBody<B>>;
  type Error = Error;
  type Transform = SecureDomiansService<S>;
  type InitError = ();
  type Future = Ready<Result<Self::Transform, Self::InitError>>;

  fn new_transform(&self, service: S) -> Self::Future {
    ready(Ok(SecureDomiansService {
      service,
      secure_domians: self.secure_domians.clone(),
    }))
  }
}

#[doc(hidden)]
pub struct SecureDomiansService<S> {
  service: S,
  secure_domians: Vec<String>,
}

impl<S> SecureDomiansService<S> {
  fn check_domian(&self, domian: String) -> bool {
    if self.secure_domians.is_empty() {
      return true;
    }
    let v: Vec<&str> = domian.split(":").collect();
    self.secure_domians.contains(&v[0].to_string())
  }
}

impl<S, B> Service<ServiceRequest> for SecureDomiansService<S>
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
  S::Future: 'static,
  B: 'static,
{
  type Response = ServiceResponse<EitherBody<B>>;
  type Error = Error;
  type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

  forward_ready!(service);

  fn call(&self, req: ServiceRequest) -> Self::Future {
    let checking = if let Some(v) = extract_referer(req.request()) {
      v
    } else {
      extract_origin(req.request())
    };
    if !self.check_domian(checking) {
      return Box::pin(async {
        Ok(req.into_response(HttpResponse::Forbidden().finish().map_into_right_body()))
      });
    }
    self
      .service
      .call(req)
      .map_ok(ServiceResponse::map_into_left_body)
      .boxed_local()
  }
}
