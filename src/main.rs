#![feature(type_alias_impl_trait)]

use crate::model::state::{SseCleanupLoop, State};
pub use crate::auth::Auth;

use std::sync::Arc;
use std::{io, env};

use actix_governor::{GovernorConfig, PeerIpKeyExtractor};
use actix_governor::governor::middleware::NoOpMiddleware;
use actix_governor::governor::clock::QuantaInstant;
use actix_web::{App, HttpServer};
use actix_web::web::Data;
use tokio::sync::RwLock;

mod model;
mod routes;
mod cors;
mod auth;
mod logger;

pub type AppState = Arc<RwLock<State>>;
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

  logger::init();
  log::info!("Starting server on port {}", PORT);

  let state = State::new();
  let state = Arc::new(RwLock::new(state));
  state.start_cleanup_loop();

  let state2 = state.clone();
  let mut state_ = state2.write().await;

  state_.this = Some(state.clone());
  drop(state_);

  let ip = env::var("IP").unwrap_or_else(|_| "0.0.0.0".to_string());

  HttpServer::new(move || {
    App::new()
      .app_data(Data::new(state.clone()))
      .wrap(cors::Cors)
      .service(routes::routes())
  })
  .bind((ip, PORT))?
  .run()
  .await
}
