use super::state::Activity;

use serde::{Serialize, Deserialize};
use serde::ser::{Serializer, SerializeStruct, SerializeSeq};

#[derive(Serialize, Deserialize)]
pub struct ActorClient {
  pub id: u32,
  pub token: String,
  pub name: String,
  pub has_access: bool,
  pub activity: Activity,
}

pub struct ActorWrapper<'a> {
  pub actor: &'a ActorClient,
  pub minimal: bool,
}

impl<'a> Serialize for ActorWrapper<'a> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    let mut state = serializer.serialize_struct("Actor", 5)?;
    state.serialize_field("id", &self.actor.id)?;
    state.serialize_field("name", &self.actor.name)?;
    
    if !self.minimal {
      state.serialize_field("has_access", &self.actor.has_access)?;
      state.serialize_field("activity", &self.actor.activity)?;
    }

    state.end()
  }
}

pub struct ActorsWrapper<'a> {
  pub actors: &'a Vec<ActorClient>,
  pub minimal: bool,
}

pub trait Wrap {
  fn wrap(&self, minimal: bool) -> ActorsWrapper;
}

impl Wrap for Vec<ActorClient> {
  fn wrap(&self, minimal: bool) -> ActorsWrapper {
    ActorsWrapper { actors: self, minimal, }
  }
}

impl<'a> Serialize for ActorsWrapper<'a> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    let mut seq = serializer.serialize_seq(Some(self.actors.len()))?;
    for actor in self.actors {
      let wrapper = ActorWrapper { actor, minimal: self.minimal };
      seq.serialize_element(&wrapper)?;
    }

    seq.end()
  }
}