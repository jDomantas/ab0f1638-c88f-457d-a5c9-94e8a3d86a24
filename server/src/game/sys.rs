//! Low level bindings to wasm game module. API corresponds 1-to-1 with wasm
//! module, except for ownership - handles are taken by reference where
//! corresponding wasm functions don't take ownership of the passed handle.

use wasmi::{self, Error};

#[derive(Debug)]
pub struct Handle {
    ptr: u32,
}

trait AsWasmValue {
    fn as_wasm_value(&self) -> wasmi::RuntimeValue;
}

impl AsWasmValue for Handle {
    fn as_wasm_value(&self) -> wasmi::RuntimeValue {
        wasmi::RuntimeValue::from(self.ptr)
    }
}

impl AsWasmValue for u32 {
    fn as_wasm_value(&self) -> wasmi::RuntimeValue {
        wasmi::RuntimeValue::from(*self)
    }
}

trait FromWasmValue: Sized {
    fn from_wasm_value(value: Option<wasmi::RuntimeValue>) -> Option<Self>;
}

impl FromWasmValue for Handle {
    fn from_wasm_value(value: Option<wasmi::RuntimeValue>) -> Option<Self> {
        u32::from_wasm_value(value).map(|ptr| Handle { ptr })
    }
}

impl FromWasmValue for u32 {
    fn from_wasm_value(value: Option<wasmi::RuntimeValue>) -> Option<Self> {
        if let Some(wasmi::RuntimeValue::I32(value)) = value {
            Some(value as u32)
        } else {
            None
        }
    }
}

impl FromWasmValue for () {
    fn from_wasm_value(value: Option<wasmi::RuntimeValue>) -> Option<Self> {
        if value.is_none() {
            Some(())
        } else {
            None
        }
    }
}

pub struct Module {
    instance: wasmi::ModuleRef,
    memory: wasmi::MemoryRef,
}

macro_rules! call {
    ($instance:expr, $name:ident ($($arg:expr),*) as $return_ty:ty) => {{
        trace!(concat!("calling wasm: ", stringify!($name)));
        let wasm_value = $instance.invoke_export(
                stringify!($name),
                &[$($arg.as_wasm_value(),)*],
                &mut wasmi::NopExternals,
            )
            .expect(concat!("failed to execute `", stringify!($name), "`"));
        <$return_ty as FromWasmValue>::from_wasm_value(wasm_value)
            .unwrap_or_else(|| {
                panic!(
                    "`{}` returned {:?}, cannot convert to {}",
                    stringify!($name),
                    wasm_value,
                    stringify!($return_ty),
                )
            })
    }}
}

impl Module {
    pub fn from_buffer(buffer: &[u8]) -> Result<Module, Error> {
        let module = wasmi::Module::from_buffer(buffer)?;
        module.deny_floating_point()?;
        let instance =
            wasmi::ModuleInstance::new(&module, &wasmi::ImportsBuilder::default())?
            .assert_no_start();
        let memory_export = instance.export_by_name("memory")
            .expect("module does not export memory");
        let memory = if let wasmi::ExternVal::Memory(memory) = memory_export {
            memory
        } else {
            panic!("`memory` export is not memory");
        };
        Ok(Module { instance, memory })
    }

    pub fn initial_world(&self) -> Handle {
        call!(self.instance, initial_world() as Handle)
    }

    pub fn update_world(&self, world: &Handle) -> Handle {
        call!(self.instance, update_world(world) as Handle)
    }

    pub fn update_player(&self, world: &Handle, player_id: u32, input: &Handle) -> Handle {
        call!(self.instance, update_player(world, player_id, input) as Handle)
    }

    pub fn add_player(&self, world: &Handle, player_id: u32) -> Handle {
        call!(self.instance, add_player(world, player_id) as Handle)
    }

    pub fn remove_player(&self, world: &Handle, player_id: u32) -> Handle {
        call!(self.instance, remove_player(world, player_id) as Handle)
    }

    pub fn allocate_buffer(&self, size: u32) -> Handle {
        call!(self.instance, allocate_buffer(size) as Handle)
    }

    pub fn free_handle(&self, handle: Handle) {
        call!(self.instance, free_handle(handle) as ())
    }

    pub fn buffer_ptr(&self, buffer: &Handle) -> u32 {
        call!(self.instance, buffer_ptr(buffer) as u32)
    }

    pub fn buffer_size(&self, buffer: &Handle) -> u32 {
        call!(self.instance, buffer_size(buffer) as u32)
    }

    pub fn serialize_world(&self, world: &Handle) -> Handle {
        call!(self.instance, serialize_world(world) as Handle)
    }

    pub fn deserialize_input(&self, buffer: &Handle) -> Handle {
        call!(self.instance, deserialize_input(buffer) as Handle)
    }

    pub fn serialize_input(&self, input: &Handle) -> Handle {
        call!(self.instance, serialize_input(input) as Handle)
    }

    pub fn write_memory(&self, ptr: u32, data: &[u8]) {
        let ptr = ptr as usize;
        self.with_memory(|memory| {
            memory[ptr..(ptr + data.len())].copy_from_slice(data);
        });
    }

    pub fn read_memory(&self, ptr: u32, size: u32, into: &mut Vec<u8>) {
        let from = ptr as usize;
        let to = from + size as usize;
        self.with_memory(|memory| {
            into.extend_from_slice(&memory[from..to]);
        });
    }

    fn with_memory<R, F: FnOnce(&mut [u8]) -> R>(&self, f: F) -> R {
        self.memory.with_direct_access_mut(|memory| f(memory))
    }
}
