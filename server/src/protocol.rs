use std::collections::HashMap;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Update {
    pub new_players: Vec<u64>,
    pub removed_players: Vec<u64>,
    pub inputs: HashMap<u64, Vec<u8>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct World {
    pub frame: u64,
    pub world: Vec<u8>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ClientMessage {
    Join { frame: u64 },
    Input { frame: u64, input: Vec<u8> },
}

pub fn world_to_json(world: &World) -> String {
    serde_json::to_string(&world).expect("failed to serialize")
}

pub fn update_to_json(update: &Update) -> String {
    serde_json::to_string(&update).expect("failed to serialize")
}

pub struct DeserializeError;

pub fn message_from_json(json: &str) -> Result<ClientMessage, DeserializeError> {
    serde_json::from_str(json).map_err(|_| DeserializeError)
}
