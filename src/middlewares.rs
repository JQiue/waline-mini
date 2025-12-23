use std::future::{Ready, ready};

use actix_web::{
  Error, HttpResponse,
  body::EitherBody,
  dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures_util::{FutureExt as _, TryFutureExt as _, future::LocalBoxFuture};

use crate::helpers::header::{extract_origin, extract_referer};

#[derive(Clone, Debug)]
pub struct SecureDomains {
  secure_domains: Vec<String>,
}

impl SecureDomains {
  pub fn new(secure_domains: Vec<String>) -> Self {
    Self { secure_domains }
  }
}

impl<S, B> Transform<S, ServiceRequest> for SecureDomains
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
  S::Future: 'static,
  B: 'static,
{
  type Response = ServiceResponse<EitherBody<B>>;
  type Error = Error;
  type Transform = SecureDomainsService<S>;
  type InitError = ();
  type Future = Ready<Result<Self::Transform, Self::InitError>>;

  fn new_transform(&self, service: S) -> Self::Future {
    ready(Ok(SecureDomainsService {
      service,
      secure_domains: self.secure_domains.clone(),
    }))
  }
}

#[doc(hidden)]
pub struct SecureDomainsService<S> {
  service: S,
  secure_domains: Vec<String>,
}

impl<S> SecureDomainsService<S> {
  fn check_domain(&self, domain: String) -> bool {
    if self.secure_domains.is_empty() {
      return true;
    }
    let v: Vec<&str> = domain.split(":").collect();
    self.secure_domains.contains(&v[0].to_string())
  }
}

impl<S, B> Service<ServiceRequest> for SecureDomainsService<S>
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
    if !self.check_domain(checking) {
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
