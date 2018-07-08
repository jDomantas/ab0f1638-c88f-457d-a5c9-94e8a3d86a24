// Allow unused function arguments.
#![allow(unused_variables)]

#[derive(Debug, Clone)]
pub struct World {
    frame: u64,
}

#[derive(Debug, Clone)]
pub struct Input {
    raw: Vec<u8>,
}

#[derive(PartialEq, Eq, Ord, PartialOrd, Debug, Hash, Clone)]
pub struct PlayerId {
    id: u64,
}

impl PlayerId {
    pub fn to_u64(&self) -> u64 {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct Buffer {
    data: Vec<u8>,
}

pub struct Game {
    next_player_id: u64,
}

impl Game {
    pub fn new() -> Game {
        Game { next_player_id: 0 }
    }

    pub fn initial_world(&mut self) -> World {
        World { frame: 0 }
    }

    pub fn update_world(&mut self, world: &World) -> World {
        World {
            frame: world.frame + 1,
        }
    }

    pub fn update_player(&mut self, player: &PlayerId, input: &Input, world: &World) -> World {
        world.clone()
    }

    pub fn remove_player(&mut self, player: &PlayerId, world: &World) -> World {
        world.clone()
    }

    pub fn add_player(&mut self, player: &PlayerId, world: &World) -> World {
        world.clone()
    }

    pub fn deserialize_input(&mut self, data: &[u8]) -> Option<Input> {
        Some(Input { raw: data.to_vec() })
    }

    pub fn serialize_world(&mut self, world: &World) -> Buffer {
        Buffer {
            data: format!("{:?}", world).into_bytes(),
        }
    }

    pub fn serialize_input(&mut self, input: &Input) -> Buffer {
        Buffer {
            data: input.raw.clone(),
        }
    }

    pub fn buffer_data<'a>(&'a self, buffer: &'a Buffer) -> &'a [u8] {
        &buffer.data
    }

    pub fn generate_player_id(&mut self) -> PlayerId {
        let id = PlayerId {
            id: self.next_player_id,
        };
        self.next_player_id += 1;
        id
    }
}
