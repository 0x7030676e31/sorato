use crate::AppState;

use std::future::{ready, Ready};
use std::env;

use actix_web::{web, error, Error};
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::middleware::Compat;
use futures_util::future::LocalBoxFuture;

pub struct Auth (bool);

impl Auth {
  pub fn new() -> Compat<Self> {
    Compat::new(Self(false))
  }

  pub fn head() -> Compat<Self> {
    Compat::new(Self(true))
  }

  pub fn new_raw() -> Self {
    Self(false)
  }

  pub fn head_raw() -> Self {
    Self(true)
  }
}

impl<S, B> Transform<S, ServiceRequest> for Auth
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
  S::Future: 'static,
  B: 'static,
{
  type Response = ServiceResponse<B>;
  type Error = Error;
  type InitError = ();
  type Transform = AuthMiddleware<S>;
  type Future = Ready<Result<Self::Transform, Self::InitError>>;

  fn new_transform(&self, service: S) -> Self::Future {
    ready(Ok(AuthMiddleware { service, head: self.0.then(|| env::var("HEAD_TOKEN").unwrap()) }))
  }
}

pub struct AuthMiddleware<S> {
  service: S,
  head: Option<String>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
  S::Future: 'static,
  B: 'static,
{
  type Response = ServiceResponse<B>;
  type Error = Error;
  type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

  forward_ready!(service);

  fn call(&self, req: ServiceRequest) -> Self::Future {
    let auth = match req.headers().get("Authorization") {
      Some(auth) => auth,
      None => return Box::pin(async { Err(error::ErrorUnauthorized("Unauthorized")) }),
    };

    let auth = match auth.to_str() {
      Ok(auth) => auth.to_owned(),
      Err(_) => return Box::pin(async { Err(error::ErrorUnauthorized("Unauthorized")) }),
    };

    if let Some(head) = &self.head {
      if &auth == head {
        return Box::pin(self.service.call(req));
      } else {
        return Box::pin(async { Err(error::ErrorUnauthorized("Unauthorized")) });
      }
    }

    let state = req.app_data::<web::Data<AppState>>().unwrap().clone();
    let fut = self.service.call(req);

    Box::pin(async move {
      let state = state.read().await;
      if !state.is_authorized(&auth) {
        return Err(error::ErrorUnauthorized("Unauthorized"));
      }
      
      Ok(fut.await?)
    })
  }
}
