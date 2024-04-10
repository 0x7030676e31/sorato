use crate::AppState;

use std::sync::OnceLock;
use std::{env, fs};

use serde::{Deserialize, Serialize};
use rand::{distributions, Rng};
use bincode::{deserialize_from, serialize_into};
use tokio::time;

const TOKEN_LENGTH: usize = 64;

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
    if !fs::metadata(&path).is_ok() {
      fs::create_dir_all(&path).unwrap();
    }

    path
  })
}

fn create_token(length: usize) -> String {
  rand::thread_rng().sample_iter(distributions::Alphanumeric).take(length).map(char::from).collect::<String>()
}

#[derive(Serialize, Deserialize)]
pub struct ActorClient {
  pub id: u64,
  pub token: String,
  pub name: String,
  pub has_access: bool,
}

#[derive(Serialize, Deserialize, Default)]
pub struct State {
  pub actors: Vec<ActorClient>,
  pub next_id: u64,

  #[serde(skip)]
  pub this: Option<AppState>,
  #[serde(skip)]
  pub codes: Vec<(String, String)>, // (code, actor name)
}

impl State {
  pub fn new() -> Self {
    let path = format!("{}/state.bin", path());
    let bytes = match fs::read(&path) {
      Ok(bytes) => bytes,
      Err(_) => return State::default()
    };

    match deserialize_from(&bytes[..]) {
      Ok(state) => state,
      Err(_) => {
        log::warn!("Failed to deserialize state, maybe the file is corrupted?");
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

  pub fn next_id(&mut self) -> u64 {
    let id = self.next_id;
    self.next_id += 1;

    id
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

  pub fn exchange_code(&mut self, code: &str) -> Option<String> {
    let index = self.codes.iter().position(|(c, _)| c == code)?;
    let (_, name) = self.codes.remove(index);

    let token = create_token(TOKEN_LENGTH);
    let id = self.next_id();

    self.actors.push(ActorClient { id, token: token.clone(), name, has_access: true });
    log::info!("Exchanged code for token {}", token);

    self.write();
    Some(token)
  }

  pub fn rename_actor(&mut self, id: u64, name: String) -> Option<()> {
    let actor = self.actors.iter_mut().find(|actor| actor.id == id)?;
    log::info!("Renamed actor {} to {}", actor.name, name);
    actor.name = name;

    self.write();
    Some(())
  }

  pub fn revoke_actor_access(&mut self, id: u64) -> Option<()> {
    let index = self.actors.iter().position(|actor| actor.id == id)?;
    log::info!("Revoked access for actor {}", self.actors[index].name);
    self.actors.remove(index);

    self.write();
    Some(())
  }

  pub fn set_actor_access(&mut self, id: u64, access: bool) -> Option<()> {
    let actor = self.actors.iter_mut().find(|actor| actor.id == id)?;
    log::info!("Set access for actor {} to {}", actor.name, access);
    actor.has_access = access;

    self.write();
    Some(())
  }
}
