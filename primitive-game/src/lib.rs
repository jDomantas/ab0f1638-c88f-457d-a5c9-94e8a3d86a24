mod game;
mod game_instance;
mod squares;

use std::sync::{Mutex, MutexGuard};
use crate::game_instance::GameInstance;

type GameImpl = crate::squares::Squares;
static mut GAME: Option<Mutex<GameInstance<GameImpl>>> = None;

fn get_game() -> MutexGuard<'static, GameInstance<GameImpl>> {
    unsafe {
        GAME.as_ref().expect("game not initialized").lock().unwrap()
    }
}

#[repr(transparent)]
pub struct Handle(pub u32);

#[no_mangle]
pub extern fn initialize() {
    unsafe {
        GAME = Some(Mutex::new(GameInstance::new()));
    }
    std::panic::set_hook(Box::new(|info| {
        log(&info.to_string());
    }));
}

#[no_mangle]
pub extern fn initial_world() -> Handle {
    get_game().initial_world()
}

#[no_mangle]
pub extern fn update_world(world: Handle) -> Handle {
    get_game().update_world(world)
}

#[no_mangle]
pub extern fn update_player(world: Handle, player_id: u32, input: Handle) -> Handle {
    get_game().update_player(world, player_id, input)
}

#[no_mangle]
pub extern fn add_player(world: Handle, player_id: u32) -> Handle {
    get_game().add_player(world, player_id)
}

#[no_mangle]
pub extern fn remove_player(world: Handle, player_id: u32) -> Handle {
    get_game().remove_player(world, player_id)
}

#[no_mangle]
pub extern fn allocate_buffer(size: u32) -> Handle {
    get_game().allocate_buffer(size)
}

#[no_mangle]
pub extern fn free_handle(handle: Handle) {
    get_game().free_handle(handle)
}

#[no_mangle]
pub extern fn buffer_ptr(buffer: Handle) -> u32 {
    get_game().buffer_ptr(buffer)
}

#[no_mangle]
pub extern fn buffer_size(buffer: Handle) -> u32 {
    get_game().buffer_size(buffer)
}

#[no_mangle]
pub extern fn deserialize_world(buffer: Handle) -> Handle {
    get_game().deserialize_world(buffer)
}

#[no_mangle]
pub extern fn serialize_world(world: Handle) -> Handle {
    get_game().serialize_world(world)
}

#[no_mangle]
pub extern fn deserialize_input(buffer: Handle) -> Handle {
    get_game().deserialize_input(buffer)
}

#[no_mangle]
pub extern fn serialize_input(input: Handle) -> Handle {
    get_game().serialize_input(input)
}

#[no_mangle]
pub extern fn create_input(letters: u32, old_letters: u32, other: u32, old_other: u32) -> Handle {
    get_game().create_input(letters, old_letters, other, old_other)
}

#[no_mangle]
pub extern fn render(world: Handle, local_player: u32, width: u32, height: u32) {
    draw_rectangle(0, 0, width, height, 0xFFFFFF);
    get_game().render(world, local_player, width, height);
}

fn draw_rectangle(x: i32, y: i32, width: u32, height: u32, color: u32) {
    unsafe {
        externals::draw_rectangle(x, y, width, height, color);
    }
}

fn log(message: &str) {
    unsafe {
        externals::log_str(message.as_ptr() as usize as u32, message.len() as u32);
    }
}

mod externals {
    extern {
        pub fn draw_rectangle(x: i32, y: i32, width: u32, height: u32, color: u32);
        pub fn log_str(ptr: u32, size: u32);
    }
}
