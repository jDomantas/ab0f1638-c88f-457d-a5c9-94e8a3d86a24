pub mod sys;

use std::ops::Deref;
use std::rc::Rc;
use self::sys::{Handle, Module};
use super::{DeserializeError, Game, ToBlob};

struct AutoHandle {
    raw: Option<Handle>,
    module: Rc<Module>,
}

impl Drop for AutoHandle {
    fn drop(&mut self) {
        self.module.free_handle(self.raw.take().unwrap());
    }
}

impl Deref for AutoHandle {
    type Target = Handle;

    fn deref(&self) -> &Self::Target {
        self.raw.as_ref().unwrap()
    }
}

pub struct World {
    handle: AutoHandle,
}

impl ToBlob for World {
    fn to_blob(&self) -> Vec<u8> {
        let buffer_handle = self.handle.module.serialize_world(&self.handle);
        let ptr = self.handle.module.buffer_ptr(&buffer_handle);
        let size = self.handle.module.buffer_size(&buffer_handle);
        let mut blob = Vec::new();
        self.handle.module.read_memory(ptr, size, &mut blob);
        self.handle.module.free_handle(buffer_handle);
        blob
    }
}

pub struct Input {
    handle: AutoHandle,
}

impl ToBlob for Input {
    fn to_blob(&self) -> Vec<u8> {
        let buffer_handle = self.handle.module.serialize_input(&self.handle);
        let ptr = self.handle.module.buffer_ptr(&buffer_handle);
        let size = self.handle.module.buffer_size(&buffer_handle);
        let mut blob = Vec::new();
        self.handle.module.read_memory(ptr, size, &mut blob);
        self.handle.module.free_handle(buffer_handle);
        blob
    }
}

#[derive(PartialEq, Eq, Ord, PartialOrd, Debug, Hash, Copy, Clone)]
pub struct PlayerId {
    id: u32,
}

impl PlayerId {
    pub fn to_u32(self) -> u32 {
        self.id
    }
}

impl Into<u64> for PlayerId {
    fn into(self) -> u64 {
        u64::from(self.to_u32())
    }
}

pub struct WasmiGame {
    next_player_id: u32,
    module: Rc<Module>,
}

impl WasmiGame {
    pub fn new(module: sys::Module) -> Self {
        Self {
            next_player_id: 0,
            module: Rc::new(module),
        }
    }
}

impl Game for WasmiGame {
    type World = World;
    type PlayerId = PlayerId;
    type Input = Input;

    fn initial_world(&mut self) -> World {
        World {
            handle: AutoHandle {
                raw: Some(self.module.initial_world()),
                module: self.module.clone(),
            },
        }
    }

    fn update_world(&mut self, world: &World) -> World {
        World {
            handle: AutoHandle {
                raw: Some(self.module.update_world(&world.handle)),
                module: self.module.clone(),
            },
        }
    }

    fn update_player(&mut self, world: &World, player: PlayerId, input: &Input) -> World {
        World {
            handle: AutoHandle {
                raw: Some(self.module.update_player(&world.handle, player.to_u32(), &input.handle)),
                module: self.module.clone(),
            },
        }
    }

    fn add_player(&mut self, world: &World, player: PlayerId) -> World {
        World {
            handle: AutoHandle {
                raw: Some(self.module.add_player(&world.handle, player.to_u32())),
                module: self.module.clone(),
            },
        }
    }

    fn remove_player(&mut self, world: &World, player: PlayerId) -> World {
        World {
            handle: AutoHandle {
                raw: Some(self.module.remove_player(&world.handle, player.to_u32())),
                module: self.module.clone(),
            },
        }
    }

    fn deserialize_input(&mut self, from: &[u8]) -> Result<Input, DeserializeError> {
        if from.len() > i32::max_value() as usize {
            panic!("buffer too large to deserialize");
        }
        let buffer_handle = self.module.allocate_buffer(from.len() as u32);
        let ptr = self.module.buffer_ptr(&buffer_handle);
        self.module.write_memory(ptr, from);
        // FIXME: somehow communicate deserialization failure
        let input = self.module.deserialize_input(&buffer_handle);
        self.module.free_handle(buffer_handle);
        Ok(Input {
            handle: AutoHandle {
                raw: Some(input),
                module: self.module.clone(),
            },
        })
    }

    fn generate_player_id(&mut self) -> PlayerId {
        let id = PlayerId { id: self.next_player_id };
        self.next_player_id += 1;
        id
    }
}
