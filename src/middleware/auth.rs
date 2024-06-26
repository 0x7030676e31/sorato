use crate::AppState;

use std::future::{ready, Ready};
use std::env;

use actix_web::{web, error, Error};
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use futures_util::future::LocalBoxFuture;

// (both?, head?)
pub struct Auth (bool, bool);

impl Auth {
  pub fn new() -> Self {
    Self(false, false)
  }

  pub fn head() -> Self {
    Self(false, true)
  }

  pub fn both() -> Self {
    Self(true, true)
  }
}

impl Default for Auth {
  fn default() -> Self {
    Self::new()
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
    ready(Ok(AuthMiddleware { service, both: self.0, head: (self.0 || self.1).then(|| env::var("HEAD_TOKEN").unwrap()) }))
  }
}

pub struct AuthMiddleware<S> {
  service: S,
  both: bool,
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
      }

      if !self.both {
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
      
      drop(state);
      fut.await
    })
  }
}
