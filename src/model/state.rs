use super::stream::{HeadSsePayload, SsePayload, IntoEvent};
use super::actor::ActorClient;
use super::client::Client;
use super::audio::Audio;
use crate::AppState;

use std::collections::HashSet;
use std::error::Error;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{env, fs, io};

use serde::{Deserialize, Serialize};
use rand::{distributions, Rng};
use bincode::{deserialize_from, serialize_into};
use tokio::sync::mpsc;
use tokio::time;
use actix_web_lab::sse;
use futures::future;
use lofty::{Probe, FileType};

const TOKEN_LENGTH: usize = 64;
const CLEANUP_INTERVAL: Duration = Duration::from_secs(15);
const ACCEPTED_FORMATS: [FileType; 6] = [
  FileType::Mpeg,
  FileType::Wav,
  FileType::Flac,
  FileType::Mp4,
  FileType::Aac,
  FileType::Vorbis,
];

pub fn path() -> &'static str {
  static PATH: OnceLock<String> = OnceLock::new();
  PATH.get_or_init(|| {
    let is_production = env::var("PRODUCTION").map_or(false, |prod| prod == "true");
    if is_production {
      return String::from("/root2/");
    }

    if env::consts::OS != "linux" {
      panic!("Unsupported OS");
    }

    let path = env::var("HOME").unwrap() + "/.config/sorato";
    if fs::metadata(&path).is_err() {
      fs::create_dir_all(&path).unwrap();
    }

    path
  })
}

// * Timestamp in ms
#[derive(Serialize, Deserialize)]
pub enum Activity {
  Online(u64),
  Offline(u64),
}

impl Activity {
  pub fn offline() -> Self {
    Self::Offline(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64)
  }

  pub fn online() -> Self {
    Self::Online(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64)
  }

  pub fn set_offline(&mut self) {
    *self = Self::offline();
  }

  pub fn set_online(&mut self) {
    *self = Self::online();
  }

  pub fn is_offline(&self) -> bool {
    matches!(self, Self::Offline(_))
  }

  pub fn is_online(&self) -> bool {
    matches!(self, Self::Online(_))
  }
}

#[derive(Serialize, Deserialize)]
pub struct Group {
  pub id: u32,
  pub name: String,
  pub members: HashSet<u32>,
}

fn create_token(length: usize) -> String {
  rand::thread_rng().sample_iter(distributions::Alphanumeric).take(length).map(char::from).collect::<String>()
}

#[derive(Serialize, Deserialize, Default)]
pub struct State {
  pub next_id: u32,
  pub actors: Vec<ActorClient>,
  pub clients: Vec<Client>,
  pub library: Vec<Audio>,
  pub groups: Vec<Group>,
  pub loader_version: u32,
  pub module_version: u32,
  pub client_version: u32,

  #[serde(skip)]
  pub this: Option<AppState>,
  #[serde(skip)]
  pub next_ack: u64,
  #[serde(skip)]
  pub codes: Vec<(String, String)>, // (code, actor name)
  #[serde(skip)]
  pub actor_stream: Vec<(u32, mpsc::Sender<sse::Event>)>,
  #[serde(skip)]
  pub head_stream: Vec<mpsc::Sender<sse::Event>>,
}

impl State {
  pub fn new() -> Self {
    let path = format!("{}/state.bin", path());
    let reader = match fs::File::open(path) {
      Ok(reader) => reader,
      Err(_) => return State::default()
    };

    match deserialize_from(reader) {
      Ok(state) => state,
      Err(err) => {
        log::warn!("Failed to deserialize state, maybe the file is corrupted? {}", err);
        State::default()
      }
    }
  }

  pub fn write(&self) {
    let path = format!("{}/state.bin", path());
    let writer = match fs::File::create(&path) {
      Ok(writer) => writer,
      Err(err) => {
        log::error!("Failed to create state file: {}", err);
        return;
      }
    };

    match serialize_into(writer, self) {
      Ok(_) => log::debug!("State written to {}", path),
      Err(err) => log::error!("Failed to write state: {}", err)
    }
  }

  pub fn next_id(&mut self) -> u32 {
    let id = self.next_id;
    self.next_id += 1;

    id
  }

  pub fn next_ack(&mut self) -> u64 {
    let ack = self.next_ack;
    self.next_ack += 1;

    ack
  }

  pub fn is_authorized(&self, token: &str) -> bool {
    self.actors.iter().any(|actor| actor.token == token && actor.has_access)
  }

  pub fn create_code(&mut self, name: String) -> String {
    let code = create_token(16);
    self.codes.push((code.clone(), name));
    log::info!("Created code {}", code);

    let code_ = code.clone();
    let this = self.this.clone().unwrap();
    tokio::spawn(async move {
      time::sleep(time::Duration::from_secs(60 * 5)).await;
      let mut state = this.write().await;
    
      if let Some(index) = state.codes.iter().position(|(c, _)| c == &code_) {
        log::info!("Code {} expired", code_);
        state.codes.remove(index);
        state.write();
      }
    });
    
    code
  }

  pub async fn exchange_code(&mut self, code: &str) -> Option<String> {
    let index = self.codes.iter().position(|(c, _)| c == code)?;
    let (_, name) = self.codes.remove(index);

    let token = create_token(TOKEN_LENGTH);
    let id = self.next_id();

    self.actors.push(ActorClient { id, token: token.clone(), name, has_access: true, activity: Activity::offline() });
    log::info!("Exchanged code for token {}", token);

    self.write();

    let ack = self.next_ack();
    let payload = HeadSsePayload::ActorCreated(self.actors.last().unwrap()).into_event(ack, None);
    self.broadcast_to_head_raw(payload).await;

    Some(token)
  }

  pub fn token_to_id(&self, token: &str) -> Option<u32> {
    self.actors.iter().find(|actor| actor.token == token).map(|actor| actor.id)
  }

  pub async fn rename_actor(&mut self, id: u32, name: String) -> Option<()> {
    let ack = self.next_ack();
    let actor = self.actors.iter_mut().find(|actor| actor.id == id)?;
    
    log::info!("Renamed actor {} to {}", actor.name, name);
    actor.name = name;
    
    let payload = HeadSsePayload::ActorRenamed(id, &actor.name).into_event(ack, None);
    
    self.write();
    self.broadcast_to_head_raw(payload).await;

    Some(())
  }

  pub async fn revoke_actor_access(&mut self, id: u32) -> Option<()> {
    let index = self.actors.iter().position(|actor| actor.id == id)?;
    log::info!("Revoked access for actor {}", self.actors[index].name);
    self.actors.remove(index);

    self.write();
    
    let payload = HeadSsePayload::ActorDeleted(id);
    self.broadcast_to_head(payload, None).await;

    let payload = SsePayload::AccessRevoked;
    self.broadcast_to_actor(id, payload, None).await;

    self.actor_stream.retain(|(actor_id, _)| *actor_id != id);
    Some(())
  }

  pub async fn set_actor_access(&mut self, id: u32, access: bool) -> Option<()> {
    let actor = self.actors.iter_mut().find(|actor| actor.id == id)?;
    if actor.has_access == access {
      return Some(());
    }

    log::info!("Set access for actor {} to {}", actor.name, access);
    actor.has_access = access;

    self.write();
    
    let payload = SsePayload::AccessChanged(access);
    self.broadcast_to_actor(id, payload, None).await;

    let payload = HeadSsePayload::ActorAccessChanged(id, access);
    self.broadcast_to_head(payload, None).await;

    Some(())
  }

  pub fn get_audio_writer(&mut self, title: String, author: Option<u32>) -> io::Result<(u32, fs::File)> {
    let id = self.next_id();
    let audio = Audio::new(id, title, author);

    self.library.push(audio);
    let dir = format!("{}/audio", path());

    if fs::metadata(&dir).is_err() {
      fs::create_dir_all(&dir)?;
    }

    let writer = fs::File::create(format!("{}/{}", dir, id))?;
    Ok((id, writer))
  }

  pub fn finalize_audio_upload(&mut self, id: u32) -> Result<Option<u32>, Box<dyn Error>> {
    let path = format!("{}/audio/{}", path(), id);
    let probe = Probe::open(&path)?.guess_file_type()?;
    let format = match probe.file_type() {
      Some(format) => format,
      None => return Ok(None),
    };

    if !ACCEPTED_FORMATS.contains(&format) {
      fs::remove_file(&path)?;
      return Ok(None);
    }

    Ok(Some(fs::metadata(&path)?.len() as u32))
  }

  pub async fn broadcast_to_head(&mut self, payload: impl IntoEvent, nonce: Option<u64>) {
    let payload = payload.into_event(self.next_ack(), nonce);
    let futures = self.head_stream.iter().map(|tx| tx.send(payload.clone()));

    let results = future::join_all(futures).await;
    for result in results {
      if let Err(err) = result {
        log::warn!("Failed to send head event: {}", err);
      }
    }
  }

  pub async fn broadcast_to_head_raw(&mut self, payload: sse::Event) {
    let futures = self.head_stream.iter().map(|tx| tx.send(payload.clone()));

    let results = future::join_all(futures).await;
    for result in results {
      if let Err(err) = result {
        log::warn!("Failed to send head event: {}", err);
      }
    }
  }

  pub async fn broadcast_to_actor<'a>(&mut self, actor_id: u32, payload: SsePayload<'a>, nonce: Option<u64>) {
    let payload = payload.into_event(self.next_ack(), nonce);

    let futures = self.actor_stream.iter().filter_map(|(id, tx)| if *id == actor_id { Some(tx.send(payload.clone())) } else { None });
    let results = future::join_all(futures).await;

    for result in results {
      if let Err(err) = result {
        log::warn!("Failed to send actor event: {}", err);
      }
    }
  }

  pub async fn broadcast_to_actor_all<'a>(&mut self, payload: SsePayload<'a>, nonce: Option<u64>) {
    let payload = payload.into_event(self.next_ack(), nonce);
    let futures = self.actor_stream.iter().map(|(_, tx)| tx.send(payload.clone()));

    let results = future::join_all(futures).await;
    for result in results {
      if let Err(err) = result {
        log::warn!("Failed to send actor event: {}", err);
      }
    }
  }

  pub async fn broadcast_to_all(&mut self, payload: impl IntoEvent, nonce: Option<u64>) {
    let payload = payload.into_event(self.next_ack(), nonce);
    let futures_1 = self.actor_stream.iter().map(|(_, tx)| tx.send(payload.clone()));
    let futures_2 = self.head_stream.iter().map(|tx| tx.send(payload.clone()));

    let results = future::join_all(futures_1.chain(futures_2)).await;
    for result in results {
      if let Err(err) = result {
        log::warn!("Failed to send event: {}", err);
      }
    }
  }
}

pub trait SseCleanupLoop {
  fn start_cleanup_loop(&self);
}

impl SseCleanupLoop for AppState {
  fn start_cleanup_loop(&self) {
    let this = self.clone();
    tokio::spawn(async move {
      let mut interval = time::interval(CLEANUP_INTERVAL);
      log::info!("Cleanup loop started");

      loop {
        interval.tick().await;
        
        let mut state = this.write().await;
        let mut ids = HashSet::new();

        let payload = SsePayload::Ping.into_event(state.next_ack(), None);
        state.head_stream.retain(|tx| {
          let is_closed = tx.is_closed() || tx.try_send(payload.clone()).is_err();
          if is_closed {
            log::info!("Head disconnected");
          }

          !is_closed
        });
        
        state.actor_stream.retain(|(id, tx)| {
          let is_closed = tx.is_closed() || tx.try_send(payload.clone()).is_err();
          if is_closed {
            log::info!("Actor {} disconnected", id);
            ids.insert(*id);
          }

          !is_closed
        });

        ids.retain(|id| {
          if state.actor_stream.iter().any(|(actor_id, _)| actor_id == id) {
            return false;
          }

          match state.actors.iter_mut().find(|actor| actor.id == *id) {
            Some(actor) => {
              log::info!("Actor {} ({}) went offline", actor.id, actor.name);
              actor.activity.set_offline();
              true
            },
            None => false
          }
        });

        if !ids.is_empty() {
          let payload = HeadSsePayload::ActorsDisconnected(&ids);
          state.broadcast_to_head(payload, None).await;
          state.write();
        }
      }
    });
  }
}