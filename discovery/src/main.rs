#![no_std]
#![no_main]
// #![feature(alloc)]
// #![feature(alloc_error_handler)]
// #![feature(custom_attribute)]

// extern crate alloc;
// extern crate alloc_cortex_m;
extern crate cortex_m;
extern crate cortex_m_rt;
extern crate cortex_m_semihosting;
extern crate panic_halt;
// extern crate wasvm;

// use alloc::alloc::Layout;
// use alloc_cortex_m::CortexMHeap;
// use cortex_m::asm;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
// use wasvm::{decode_module, init_store, instantiate_module, ExternalModules, Values};

// #[global_allocator]
// static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[entry]
fn main() -> ! {
    // Need to enable semihosting of gdb
    hprintln!("Hello, world").unwrap();

    // let start = rt::heap_start() as usize;
    // let size = 1024;
    // unsafe { ALLOCATOR.init(start, size) }

    // // add.wasm
    // let bytes = [
    //     0x61, 0x00, 0x6d, 0x73, 0x00, 0x01, 0x00, 0x00, 0x07, 0x01, 0x60, 0x01, 0x7f, 0x02, 0x01,
    //     0x7f, 0x03, 0x7f, 0x01, 0x02, 0x07, 0x00, 0x01, 0x0c, 0x5f, 0x08, 0x75, 0x73, 0x6a, 0x62,
    //     0x63, 0x65, 0x00, 0x74, 0x0a, 0x00, 0x01, 0x09, 0x00, 0x07, 0x01, 0x20, 0x00, 0x20, 0x0b,
    //     0x6a,
    // ];

    loop {
        // let store = init_store();
        // let section = decode_module(&bytes);
        // let external_modules = ExternalModules::default();
        // let mut vm = instantiate_module(store, section, external_modules).unwrap();
        // let result = vm.run("_subject", [Values::I32(10), Values::I32(20)].to_vec());
    }
}

// #[alloc_error_handler]
// fn on_oom(_layout: Layout) -> ! {
//     asm::bkpt();

//     loop {}
// }
