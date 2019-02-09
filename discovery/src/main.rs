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
use alloc_cortex_m::CortexMHeap;
use cortex_m::asm;
use cortex_m_rt::{entry, heap_start};
use cortex_m_semihosting::hprintln;
use wasvm::{decode_module, init_store, instantiate_module, ExternalModules, Values};

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[entry]
fn main() -> ! {
    let start = heap_start() as usize;
    let size = 65536;
    unsafe { ALLOCATOR.init(start, size) }

    hprintln!("Start...").unwrap();

    let bytes = include_bytes!("add.wasm");
    let store = init_store();
    let section = decode_module(bytes);
    let external_modules = ExternalModules::default();
    let instance = instantiate_module(store, section, external_modules);
    let mut vm = instance.unwrap();
    let result = vm.run("_subject", [Values::I32(10), Values::I32(20)].to_vec());

    // Need to enable semihosting of gdb
    hprintln!("Result={:?}", result).unwrap();

    loop {}
}

#[alloc_error_handler]
fn on_oom(_layout: Layout) -> ! {
    asm::bkpt();

    loop {}
}
