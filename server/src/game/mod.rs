// Allow unused function arguments.
#![allow(unused_variables)]

pub mod sys;

use std::ops::Deref;
use std::rc::Rc;
use self::sys::{Handle, Module};

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

pub struct Input {
    handle: AutoHandle,
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

pub struct Game {
    next_player_id: u32,
    module: Rc<Module>,
}

impl Game {
    pub fn new(module: sys::Module) -> Game {
        Game {
            next_player_id: 0,
            module: Rc::new(module),
        }
    }

    pub fn initial_world(&mut self) -> World {
        World {
            handle: AutoHandle {
                raw: Some(self.module.initial_world()),
                module: self.module.clone(),
            },
        }
    }

    pub fn update_world(&mut self, world: &World) -> World {
        World {
            handle: AutoHandle {
                raw: Some(self.module.update_world(&world.handle)),
                module: self.module.clone(),
            },
        }
    }

    pub fn update_player(&mut self, player: PlayerId, input: &Input, world: &World) -> World {
        World {
            handle: AutoHandle {
                raw: Some(self.module.update_player(&world.handle, player.to_u32(), &input.handle)),
                module: self.module.clone(),
            },
        }
    }

    pub fn remove_player(&mut self, player: PlayerId, world: &World) -> World {
        World {
            handle: AutoHandle {
                raw: Some(self.module.remove_player(&world.handle, player.to_u32())),
                module: self.module.clone(),
            },
        }
    }

    pub fn add_player(&mut self, player: PlayerId, world: &World) -> World {
        World {
            handle: AutoHandle {
                raw: Some(self.module.add_player(&world.handle, player.to_u32())),
                module: self.module.clone(),
            },
        }
    }

    pub fn deserialize_input(&mut self, data: &[u8]) -> Option<Input> {
        if data.len() > i32::max_value() as usize {
            panic!("buffer too large to deserialize");
        }
        let buffer_handle = self.module.allocate_buffer(data.len() as u32);
        let ptr = self.module.buffer_ptr(&buffer_handle);
        self.module.write_memory(ptr, data);
        // FIXME: how to communicate deserialization failure?
        let input = self.module.deserialize_input(&buffer_handle);
        self.module.free_handle(buffer_handle);
        Some(Input {
            handle: AutoHandle {
                raw: Some(input),
                module: self.module.clone(),
            },
        })
    }

    pub fn serialize_world(&mut self, world: &World, into: &mut Vec<u8>) {
        let buffer_handle = self.module.serialize_world(&world.handle);
        let ptr = self.module.buffer_ptr(&buffer_handle);
        let size = self.module.buffer_size(&buffer_handle);
        self.module.read_memory(ptr, size, into);
    }

    pub fn serialize_input(&mut self, input: &Input, into: &mut Vec<u8>) {
        let buffer_handle = self.module.serialize_input(&input.handle);
        let ptr = self.module.buffer_ptr(&buffer_handle);
        let size = self.module.buffer_size(&buffer_handle);
        self.module.read_memory(ptr, size, into);
    }

    pub fn generate_player_id(&mut self) -> PlayerId {
        let id = PlayerId {
            id: self.next_player_id,
        };
        self.next_player_id += 1;
        id
    }
}
