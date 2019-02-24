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

#[derive(Deserialize, PartialEq, Eq, Debug)]
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

#[derive(Debug)]
pub struct DeserializeError;

pub fn message_from_json(json: &str) -> Result<ClientMessage, DeserializeError> {
    serde_json::from_str(json).map_err(|_| DeserializeError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_serialization() {
        let world = World { frame: 123, world: vec![4, 5, 6] };
        let json = world_to_json(&world);
        assert_eq!(
            json,
            r#"  {"frame":123,"world":[4,5,6]}  "#.trim(),
        );
    }

    #[test]
    fn update_serialization() {
        let update = Update {
            new_players: vec![1],
            removed_players: vec![2],
            inputs: {
                let mut map = HashMap::new();
                map.insert(3, vec![4]);
                map
            },
        };
        let json = update_to_json(&update);
        assert_eq!(
            json,
            r#"  {"newPlayers":[1],"removedPlayers":[2],"inputs":{"3":[4]}}  "#.trim(),
        );
    }

    #[test]
    fn client_join_deserialization() {
        let json = r#"
            { "join": { "frame": 123 } }
        "#;
        let msg = message_from_json(json).expect("failed to deserialize");
        assert_eq!(
            msg,
            ClientMessage::Join { frame: 123 },
        );
    }

    #[test]
    fn client_input_deserialization() {
        let json = r#"
            { "input": { "frame": 123, "input": [4, 5, 6] } }
        "#;
        let msg = message_from_json(json).expect("failed to deserialize");
        assert_eq!(
            msg,
            ClientMessage::Input { frame: 123, input: vec![4, 5, 6] },
        );
    }
}
