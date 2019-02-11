extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    #[no_mangle]
    pub fn my_hal_function(a: i32, b: i32) -> i32;
}

#[no_mangle]
#[wasm_bindgen]
pub fn use_hal_function(a: i32, b: i32) -> i32 {
    my_hal_function(a, b) + 10
}

