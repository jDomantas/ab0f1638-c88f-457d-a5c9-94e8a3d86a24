use crate::game::{Game, PlayerId, Serialize, KeyboardState};
use crate::Handle;

enum Object<G: Game> {
    World(G::World),
    Input(G::Input),
    Buffer(Vec<u8>),
}

impl<G: Game> Object<G> {
    fn as_world(&self) -> &G::World {
        match self {
            Object::World(world) => world,
            Object::Input(_) => panic!("expected world, got input"),
            Object::Buffer(_) => panic!("expected world, got buffer"),
        }
    }

    fn as_input(&self) -> &G::Input {
        match self {
            Object::World(_) => panic!("expected input, got world"),
            Object::Input(input) => input,
            Object::Buffer(_) => panic!("expected input, got buffer"),
        }
    }

    fn as_buffer(&mut self) -> &mut [u8] {
        match self {
            Object::World(_) => panic!("expected buffer, got world"),
            Object::Input(_) => panic!("expected buffer, got input"),
            Object::Buffer(buf) => buf,
        }
    }
}

pub struct GameInstance<G: Game> {
    objects: Vec<Option<Object<G>>>,
}

impl<G: Game> GameInstance<G> {
    pub fn new() -> GameInstance<G> {
        GameInstance {
            objects: Vec::new(),
        }
    }

    fn object(&self, handle: Handle) -> &Object<G> {
        self.objects
            .get(handle.0 as usize)
            .and_then(|x| x.as_ref())
            .expect("empty handle")
    }

    fn buffer_mut(&mut self, handle: Handle) -> &mut [u8] {
        self.objects
            .get_mut(handle.0 as usize)
            .and_then(|x| x.as_mut())
            .expect("empty handle (mut)")
            .as_buffer()
    }

    fn create_object(&mut self, object: Object<G>) -> Handle {
        let object = Object::from(object);
        for (index, slot) in self.objects.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(object);
                return Handle(index as u32);
            }
        }
        self.objects.push(Some(object));
        Handle((self.objects.len() - 1) as u32)
    }

    fn create_world(&mut self, world: G::World) -> Handle {
        self.create_object(Object::World(world))
    }

    pub fn initial_world(&mut self) -> Handle {
        let world = G::initial_world();
        self.create_world(world)
    }

    pub fn update_world(&mut self, world: Handle) -> Handle {
        let new_world = {
            let world = self.object(world).as_world();
            G::update_world(world)
        };
        self.create_world(new_world)
    }

    pub fn update_player(&mut self, world: Handle, player_id: u32, input: Handle) -> Handle {
        let new_world = {
            let world = self.object(world).as_world();
            let input = self.object(input).as_input();
            G::update_player(world, PlayerId::new(player_id), input)
        };
        self.create_world(new_world)
    }

    pub fn add_player(&mut self, world: Handle, player_id: u32) -> Handle {
        let new_world = {
            let world = self.object(world).as_world();
            G::add_player(world, PlayerId::new(player_id))
        };
        self.create_world(new_world)
    }

    pub fn remove_player(&mut self, world: Handle, player_id: u32) -> Handle {
        let new_world = {
            let world = self.object(world).as_world();
            G::remove_player(world, PlayerId::new(player_id))
        };
        self.create_world(new_world)
    }

    pub fn allocate_buffer(&mut self, size: u32) -> Handle {
        let buffer = vec![0u8; size as usize];
        self.create_object(Object::Buffer(buffer))
    }

    pub fn buffer_ptr(&mut self, buffer: Handle) -> u32 {
        self.buffer_mut(buffer).as_ptr() as usize as u32
    }

    pub fn buffer_size(&mut self, buffer: Handle) -> u32 {
        self.buffer_mut(buffer).len() as u32
    }

    pub fn free_handle(&mut self, handle: Handle) {
        self.objects[handle.0 as usize] = None;
    }

    pub fn deserialize_world(&mut self, buffer: Handle) -> Handle {
        let world = {
            let mut buffer = &self.buffer_mut(buffer)[..];
            G::World::read(&mut buffer).expect("failed to deserialize world")
        };
        self.create_world(world)
    }

    pub fn deserialize_input(&mut self, buffer: Handle) -> Handle {
        let input = {
            let mut buffer = &self.buffer_mut(buffer)[..];
            G::Input::read(&mut buffer).expect("failed to deserialize input")
        };
        self.create_object(Object::Input(input))
    }

    pub fn serialize_world(&mut self, world: Handle) -> Handle {
        let mut buf = Vec::new();
        {
            let world = self.object(world).as_world();
            world.write(&mut buf);
        }
        self.create_object(Object::Buffer(buf))
    }

    pub fn serialize_input(&mut self, input: Handle) -> Handle {
        let mut buf = Vec::new();
        {
            let input = self.object(input).as_input();
            input.write(&mut buf);
        }
        self.create_object(Object::Buffer(buf))
    }

    pub fn create_input(&mut self, letters: u32, old_letters: u32, other: u32, old_other: u32) -> Handle {
        let keyboard_state = KeyboardState::new(letters, old_letters, other, old_other);
        let input = G::create_input(keyboard_state);
        self.create_object(Object::Input(input))
    }

    pub fn render(&mut self, world: Handle, local_player: u32, width: u32, height: u32) {
        let world = self.object(world).as_world();
        G::render(world, PlayerId::new(local_player), width, height);
    }
}
