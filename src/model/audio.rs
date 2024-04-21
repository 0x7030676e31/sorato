use std::collections::HashSet;

use serde::{Serialize, Deserialize};
use chrono::Utc;

#[derive(Serialize, Deserialize)]
pub struct Audio {
  pub id: u32,
  pub title: String,
  pub length: u32, // in ms
  pub downloads: HashSet<u32>,
  pub author: Option<u32>,
  pub created: u64, // in ms
}

impl Audio {
  pub fn new(id: u32, title: String, author: Option<u32>) -> Self {
    Self {
      id,
      title,
      length: 0,
      downloads: HashSet::new(),
      author,
      created: Utc::now().timestamp_millis() as u64,
    }
  }
}
