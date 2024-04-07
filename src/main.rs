use std::sync::Arc;
use std::{io, env};

use model::state::State;

use actix_web::{App, HttpServer};
use actix_web::web::Data;
use tokio::sync::RwLock;

mod model;
mod routes;
mod cors;

type AppState = Arc<RwLock<State>>;
const PORT: u16 = 8080;

#[tokio::main]
async fn main() -> io::Result<()> {
  if env::var("ACTOR_TOKEN").is_err() {
    panic!("Missing ACTOR_TOKEN environment variable");
  }
  
  if env::var("RUST_LOG").is_err() {
    env::set_var("RUST_LOG", "INFO");
  }

  pretty_env_logger::init();

  log::info!("Starting server on port {}", PORT);

  let state = Arc::new(RwLock::new(State::new()));
  let ip = env::var("IP").unwrap_or_else(|_| "0.0.0.0".to_string());

  HttpServer::new(move || {
    App::new()
      .app_data(Data::new(state.clone()))
      .service(routes::routes())
  })
  .bind((ip, PORT))?
  .run()
  .await
}
