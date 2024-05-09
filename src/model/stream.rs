use std::collections::HashSet;

use super::actor::{ActorClient, ActorsWrapper};
use super::audio::Audio;
use super::client::ClientsWrapper;

use serde::Serialize;
use actix_web_lab::sse::{Event, Data};

pub trait IntoEvent {
  fn into_event(self, ack: u64, nonce: Option<u64>) -> Event;
}

#[derive(Serialize)]
#[serde(tag = "type", content = "payload")]
#[allow(dead_code)]
pub enum SsePayload<'a> {
  Ready {
    clients: ClientsWrapper<'a>,
    actors: ActorsWrapper<'a>,
    library: &'a Vec<Audio>,
    has_access: bool,
    id: u32,
  },
  Ping,
  AccessChanged(bool),
  AccessRevoked,
  AudioCreated {
    id: u32,
    title: &'a str,
    length: u32,
    author: Option<u32>,
  },
  AudioDeleted(u32),
}

#[derive(Serialize)]
#[serde(tag = "type", content = "payload")]
#[allow(dead_code)]
pub enum HeadSsePayload<'a> {
  ReadyHead {
    clients: ClientsWrapper<'a>,
    actors: ActorsWrapper<'a>,
    library: &'a Vec<Audio>,
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

impl<'a> IntoEvent for SsePayload<'a> {
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

