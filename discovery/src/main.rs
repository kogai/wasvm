#![no_std]
#![no_main]
#![feature(alloc)]
#![feature(alloc_error_handler)]
#![feature(custom_attribute)]
#![feature(core_intrinsics)]

extern crate alloc;
extern crate alloc_cortex_m;
extern crate cortex_m;
extern crate cortex_m_rt;
extern crate cortex_m_semihosting;
extern crate panic_halt;
extern crate wasvm;

use alloc::alloc::Layout;
use alloc::prelude::*;
use alloc_cortex_m::CortexMHeap;
use cortex_m::asm;
use cortex_m_rt::{entry, heap_start};
use cortex_m_semihosting::hprintln;
use wasvm::{
    decode_module, init_store, instantiate_module, ExternalModule, ExternalModules,
    FunctionInstance, FunctionType, ValueTypes, Values,
};

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

fn my_hal_function(_arguments: &[Values]) -> alloc::vec::Vec<Values> {
    [Values::I32(3 * 5)].to_vec()
}

#[entry]
fn main() -> ! {
    let start = heap_start() as usize;
    let size = 65536;
    unsafe { ALLOCATOR.init(start, size) }

    hprintln!("Start...").unwrap();

    let bytes = include_bytes!("discovery_wasm_bg.wasm");
    let store = init_store();
    let section = decode_module(bytes);
    let mut external_modules = ExternalModules::default();
    let external_module = ExternalModule::new(
        [FunctionInstance::new_host_fn(
            Some("__wbg_myhalfunction_59a89d8df8955cf7".to_owned()),
            FunctionType::new(
                [ValueTypes::I32, ValueTypes::I32].to_vec(),
                [ValueTypes::I32].to_vec(),
            ),
            &my_hal_function,
        )]
        .to_vec(),
        [].to_vec(),
        [].to_vec(),
        [].to_vec(),
        [].to_vec(),
    );
    external_modules.register_module(Some("./discovery_wasm".to_owned()), external_module).unwrap();
    // FIXME: Causes OOM.
    let instance = instantiate_module(store, section, external_modules, 128);
    let mut vm = instance.unwrap();
    let result = vm.run(
        "use_hal_function",
        [Values::I32(3), Values::I32(5)].to_vec(),
    );

    // Need to enable semihosting of gdb
    hprintln!("Result={:?}", result).unwrap();

    loop {}
}

#[alloc_error_handler]
fn on_oom(_layout: Layout) -> ! {
    asm::bkpt();

    loop {}
}
