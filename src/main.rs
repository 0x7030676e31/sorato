#![feature(type_alias_impl_trait)]

use std::sync::Arc;
use std::{io, env};

use model::state::State;
pub use auth::Auth;

use actix_governor::{GovernorConfig, PeerIpKeyExtractor};
use actix_governor::governor::middleware::NoOpMiddleware;
use actix_governor::governor::clock::QuantaInstant;
// use actix_web::{App, Error, HttpServer, Scope};
use actix_web::{App, HttpServer};
// use actix_web::body::{BoxBody, EitherBody};
use actix_web::web::Data;
// use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use tokio::sync::RwLock;

mod model;
mod routes;
mod cors;
mod auth;

pub type AppState = Arc<RwLock<State>>;
// pub type Scoped = Scope<impl ServiceFactory<ServiceRequest, Config = (), Response = ServiceResponse, Error = actix_web::Error, InitError = ()>>;
// pub type ScopedEx = Scope<impl ServiceFactory<ServiceRequest, Config = (), Response = ServiceResponse<EitherBody<BoxBody>>, Error = Error, InitError = ()>>;
pub type Secure = GovernorConfig<PeerIpKeyExtractor, NoOpMiddleware<QuantaInstant>>;

const PORT: u16 = 8080;

#[tokio::main]
async fn main() -> io::Result<()> {
  dotenv::dotenv().ok();
  if env::var("HEAD_TOKEN").is_err() {
    panic!("Missing HEAD_TOKEN environment variable");
  }
  
  if env::var("RUST_LOG").is_err() {
    env::set_var("RUST_LOG", "INFO");
  }

  pretty_env_logger::init();

  log::info!("Starting server on port {}", PORT);

  let state = State::new();
  let state = Arc::new(RwLock::new(state));

  let state2 = state.clone();
  let mut state_ = state2.write().await;

  state_.this = Some(state.clone());
  drop(state_);

  let ip = env::var("IP").unwrap_or_else(|_| "0.0.0.0".to_string());

  HttpServer::new(move || {
    App::new()
      .app_data(Data::new(state.clone()))
      // .wrap(actix_cors::Cors::permissive())
      .wrap(cors::Cors)
      .service(routes::routes())
  })
  .bind((ip, PORT))?
  .run()
  .await
}
