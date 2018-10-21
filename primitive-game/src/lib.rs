#![allow(unused_variables)]

#[repr(transparent)]
pub struct Handle(pub u32);

static DUMMY_BUFFER: &[u8] = &[1, 1, 2, 3, 5, 8, 11];

const DUMMY_HANDLE: Handle = Handle(0);

#[no_mangle]
pub extern fn initial_world() -> Handle { DUMMY_HANDLE }

#[no_mangle]
pub extern fn update_world(world: Handle) -> Handle { DUMMY_HANDLE }

#[no_mangle]
pub extern fn update_player(world: Handle, player_id: u64, input: Handle) -> Handle { DUMMY_HANDLE }

#[no_mangle]
pub extern fn add_player(world: Handle, player_id: u64) -> Handle { DUMMY_HANDLE }

#[no_mangle]
pub extern fn remove_player(world: Handle, player_id: u64) -> Handle { DUMMY_HANDLE }

#[no_mangle]
pub extern fn allocate_buffer(size: u32) -> Handle { DUMMY_HANDLE }

#[no_mangle]
pub extern fn free_handle(handle: Handle) {}

#[no_mangle]
pub extern fn buffer_ptr(buffer: Handle) -> u32 { DUMMY_BUFFER.as_ptr() as usize as u32 }

#[no_mangle]
pub extern fn buffer_size(buffer: Handle) -> u32 { DUMMY_BUFFER.len() as u32 }

#[no_mangle]
pub extern fn deserialize_world(buffer: Handle) -> Handle { DUMMY_HANDLE }

#[no_mangle]
pub extern fn serialize_world(world: Handle) -> Handle { DUMMY_HANDLE }

#[no_mangle]
pub extern fn deserialize_input(buffer: Handle) -> Handle { DUMMY_HANDLE }

#[no_mangle]
pub extern fn serialize_input(input: Handle) -> Handle { DUMMY_HANDLE }
