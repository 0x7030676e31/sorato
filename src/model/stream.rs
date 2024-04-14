use std::collections::HashSet;

use super::state::ActorClient;

use serde::Serialize;
use actix_web_lab::sse::{Event, Data};

pub trait IntoEvent {
  fn into_event(self, ack: u64, nonce: Option<u64>) -> Event;
}

#[derive(Serialize)]
#[serde(tag = "type", content = "payload")]
#[allow(dead_code)]
pub enum SsePayload {
  Ready {
    has_access: bool,
  },
  Ping,
  AccessChanged(bool),
  AccessRevoked,
}

#[derive(Serialize)]
#[serde(tag = "type", content = "payload")]
#[allow(dead_code)]
pub enum HeadSsePayload<'a> {
  ReadyHead {
    actors: &'a Vec<ActorClient>,
  },
  ActorCreated(&'a ActorClient),
  ActorConnected(u32),
  ActorsDisconnected(&'a HashSet<u32>),
  ActorRenamed(u32, &'a str),
  ActorAccessChanged(u32, bool),
  ActorDeleted(u32),
}

#[derive(Serialize)]
struct PayloadInner<T> where T: Serialize {
  payload: T,
  nonce: Option<u64>,
  ack: u64,
}

impl IntoEvent for SsePayload {
  fn into_event(self, ack: u64, nonce: Option<u64>) -> Event {
    let inner = PayloadInner { payload: self, ack, nonce, };
    Event::Data(Data::new_json(inner).unwrap())
  }
}

impl<'a> IntoEvent for HeadSsePayload<'a> {
  fn into_event(self, ack: u64, nonce: Option<u64>) -> Event {
    let inner = PayloadInner { payload: self, ack, nonce, };
    Event::Data(Data::new_json(inner).unwrap())
  }
}

