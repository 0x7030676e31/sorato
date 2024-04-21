use crate::model::stream::{HeadSsePayload, SsePayload, IntoEvent};
use crate::model::client::Wrap as _;
use crate::model::actor::Wrap as _;
use crate::AppState;

use std::env;

use actix_web::{web, HttpRequest, error, Responder, get};
use actix_web_lab::sse::Sse;
use tokio::sync::mpsc;

#[get("/stream")]
pub async fn stream(req: HttpRequest, state: web::Data<AppState>) -> Result<impl Responder, error::Error> {
  let auth = match req.headers().get("Authorization") {
    Some(auth) => auth,
    None => return Err(error::ErrorUnauthorized("Unauthorized")),
  };

  let auth = match auth.to_str() {
    Ok(auth) => auth,
    Err(_) => return Err(error::ErrorUnauthorized("Unauthorized")),
  };

  let (tx, rx) = mpsc::channel(32);
  let mut state = state.write().await;

  if env::var("HEAD_TOKEN").unwrap() == auth {
    state.head_stream.push(tx.clone());
    let ack = state.next_ack();

    let payload = HeadSsePayload::ReadyHead {
      clients: state.clients.wrap(false),
      actors: state.actors.wrap(false),
      library: &state.library,
    };

    let payload= payload.into_event(ack, None);
    if let Err(err) = tx.send(payload).await {
      log::error!("Failed to send ready event: {}", err);
    }

    log::info!("Head connected");
    return Ok(Sse::from_infallible_receiver(rx))
  }

  let actor = match state.actors.iter_mut().find(|actor| actor.token == auth) {
    Some(actor) => actor,
    None => return Err(error::ErrorUnauthorized("Unauthorized")),
  };

  let actor_id = actor.id;  
  let has_access = actor.has_access;

  if actor.activity.is_offline() {
    actor.activity.set_online();
    let payload = HeadSsePayload::ActorConnected(actor.id);

    state.broadcast_to_head(payload, None).await;
  }

  log::info!("Actor {} connected", actor_id);
  state.actor_stream.push((actor_id, tx.clone()));

  let ack = state.next_ack();
  
  let payload = SsePayload::Ready {
    clients: state.clients.wrap(false),
    actors: state.actors.wrap(true),
    library: &state.library,
    has_access,
    id: actor_id,
  };
  
  let payload = payload.into_event(ack, None);
  if let Err(err) = tx.send(payload).await {
    log::error!("Failed to send ready event: {}", err);
  }  

  Ok(Sse::from_infallible_receiver(rx))
}