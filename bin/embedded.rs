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
use alloc::vec::Vec;
use alloc_cortex_m::CortexMHeap;
use cortex_m::asm;
use wasvm::init_store;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[entry]
fn main() -> ! {
  let _store = init_store();
  let start = rt::heap_start() as usize;
  let size = 1024; // in bytes
  unsafe { ALLOCATOR.init(start, size) }

  let mut xs = Vec::new();
  xs.push(1);

  loop {
    // your code goes here
  }
}

#[cfg(not(test))]
#[alloc_error_handler]
fn on_oom(_layout: Layout) -> ! {
  asm::bkpt();

  loop {}
}
