#![allow(unused_variables)]

#[repr(transparent)]
pub struct Handle(pub i32);

const DUMMY_HANDLE: Handle = Handle(0);

#[no_mangle]
pub extern fn initial_world() -> Handle { DUMMY_HANDLE }

#[no_mangle]
pub extern fn update_world(world: Handle) -> Handle { DUMMY_HANDLE }

#[no_mangle]
pub extern fn update_player(world: Handle, player_id: Handle, input: Handle) -> Handle { DUMMY_HANDLE }

#[no_mangle]
pub extern fn add_player(world: Handle, player_id: Handle) -> Handle { DUMMY_HANDLE }

#[no_mangle]
pub extern fn remove_player(world: Handle, player_id: Handle) -> Handle { DUMMY_HANDLE }

#[no_mangle]
pub extern fn generate_player_id() -> Handle { DUMMY_HANDLE }

#[no_mangle]
pub extern fn allocate_buffer(size: i32) -> Handle { DUMMY_HANDLE }

#[no_mangle]
pub extern fn free_handle(handle: Handle) {}

#[no_mangle]
pub extern fn buffer_ptr(buffer: Handle) -> i32 { 0 }

#[no_mangle]
pub extern fn buffer_size(buffer: Handle) -> i32 { 0 }

#[no_mangle]
pub extern fn deserialize_world(buffer: Handle) -> Handle { DUMMY_HANDLE }

#[no_mangle]
pub extern fn serialize_world(world: Handle) -> Handle { DUMMY_HANDLE }

#[no_mangle]
pub extern fn deserialize_input(buffer: Handle) -> Handle { DUMMY_HANDLE }

#[no_mangle]
pub extern fn serialize_input(input: Handle) -> Handle { DUMMY_HANDLE }
