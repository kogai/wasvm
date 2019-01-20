#![no_std]
#![no_main]
#![feature(alloc)]
#![feature(alloc_error_handler)]
#![feature(custom_attribute)]

extern crate alloc;
extern crate alloc_cortex_m;
// #[macro_use]
extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate panic_halt;
extern crate wasvm;
// extern crate panic_abort; // requires nightly
// extern crate panic_itm; // logs messages over ITM; requires ITM support
// extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger

use alloc::alloc::Layout;
use alloc_cortex_m::CortexMHeap;
use cortex_m::asm;
use wasvm::{init_store, decode_module, instantiate_module, ExternalModules, Values};

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[entry]
fn main() -> ! {
    let start = rt::heap_start() as usize;
    let size = 1024;
    unsafe { ALLOCATOR.init(start, size) }

    // add.wasm
    let bytes = [
        0x61, 0x00, 0x6d, 0x73, 0x00, 0x01, 0x00, 0x00, 0x07, 0x01, 0x60, 0x01, 0x7f, 0x02, 0x01,
        0x7f, 0x03, 0x7f, 0x01, 0x02, 0x07, 0x00, 0x01, 0x0c, 0x5f, 0x08, 0x75, 0x73, 0x6a, 0x62,
        0x63, 0x65, 0x00, 0x74, 0x0a, 0x00, 0x01, 0x09, 0x00, 0x07, 0x01, 0x20, 0x00, 0x20, 0x0b,
        0x6a,
    ];
    let store = init_store();
    let section = decode_module(&bytes);
    let external_modules = ExternalModules::default();
    let mut vm = instantiate_module(store, section, external_modules).unwrap();
    vm.run("_subject", [Values::I32(10), Values::I32(20)].to_vec());

    loop {}
}

#[alloc_error_handler]
fn on_oom(_layout: Layout) -> ! {
  asm::bkpt();

  loop {}
}
