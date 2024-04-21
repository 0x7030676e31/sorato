use super::state::Activity;

use serde::{Serialize, Deserialize};
use serde::ser::{Serializer, SerializeStruct, SerializeSeq};

#[derive(Serialize, Deserialize)]
pub struct Client {
  pub id: u32,
  pub token: String,
  pub alias: String,
  pub hostname: String,
  pub username: String,
  pub last_ip: String,
  pub versions: (u32, u32, u32), // Loader, Module, Client
  pub activity: Activity,
}

struct ClientWrapper<'a> {
  client: &'a Client,
  serialize_token: bool,
}

impl<'a> Serialize for ClientWrapper<'a> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    let mut state = serializer.serialize_struct("Client", 8)?;
    state.serialize_field("id", &self.client.id)?;
    state.serialize_field("alias", &self.client.alias)?;
    state.serialize_field("hostname", &self.client.hostname)?;
    state.serialize_field("username", &self.client.username)?;
    state.serialize_field("last_ip", &self.client.last_ip)?;
    state.serialize_field("versions", &self.client.versions)?;
    state.serialize_field("activity", &self.client.activity)?;
    
    if self.serialize_token {
      state.serialize_field("token", &self.client.token)?;
    }
    
    state.end()
  }
}

pub struct ClientsWrapper<'a> {
  pub clients: &'a Vec<Client>,
  pub serialize_token: bool,
}

pub trait Wrap {
  fn wrap(&self, serialize_token: bool) -> ClientsWrapper;
}

impl Wrap for Vec<Client> {
  fn wrap(&self, serialize_token: bool) -> ClientsWrapper {
    ClientsWrapper { clients: self, serialize_token, }
  }
}

impl<'a> Serialize for ClientsWrapper<'a> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    let mut seq = serializer.serialize_seq(Some(self.clients.len()))?;
    for client in self.clients {
      let wrapper = ClientWrapper { client, serialize_token: self.serialize_token, };
      seq.serialize_element(&wrapper)?;
    }

    seq.end()
  }
}
