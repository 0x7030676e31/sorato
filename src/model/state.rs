use std::collections::HashSet;
use std::sync::OnceLock;
use std::{env, fs};

use serde::{Deserialize, Serialize};
use rand::{distributions, Rng};
use bincode::{deserialize_from, serialize_into};

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

#[derive(Serialize, Deserialize, Default)]
pub struct State {
  pub actors: HashSet<String>,

  #[serde(skip)]
  pub codes: HashSet<String>,
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

  pub fn create_code(&mut self) -> String {
    let code = create_token(16);
    self.codes.insert(code.clone());
    log::info!("Created code {}", code);

    code
  }

  pub fn exchange_code(&mut self, code: &str) -> Option<String> {
    log::info!("Exchanging code {}", code);
    if self.codes.remove(code) {
      log::info!("Code {} was exchanged", code);
      Some(self.create_actor_token())
    } else {
      log::warn!("Code {} was not found", code);
      None
    }
  }

  fn create_actor_token(&mut self) -> String {
    let token = create_token(TOKEN_LENGTH);
    self.actors.insert(token.clone());

    log::info!("Created actor token {}", token);
    self.write();

    token
  }

  pub fn revoke_actor_token(&mut self, token: &str) -> bool {
    log::info!("Revoking actor token {}", token);
    let removed = self.actors.remove(token);
    
    self.write();
    removed
  }
}
