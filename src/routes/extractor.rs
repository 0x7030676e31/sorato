use std::pin::Pin;

use actix_web::{FromRequest, HttpRequest, dev, error};
use futures::Future;

pub struct Token (pub String);

impl FromRequest for Token {
  type Error = actix_web::Error;
  type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send>>;
  
  fn from_request(req: &HttpRequest, _: &mut dev::Payload) -> Self::Future {
    let token = match req.headers().get("Authorization") {
      Some(token) => token,
      None => return Box::pin(async { Err(error::ErrorUnauthorized("Unauthorized")) }),
    };

    let token = match token.to_str() {
      Ok(token) => token,
      Err(_) => return Box::pin(async { Err(error::ErrorUnauthorized("Unauthorized")) }),
    };

    let token = token.to_string();
    Box::pin(async { Ok(Token(token)) })
  }
}
