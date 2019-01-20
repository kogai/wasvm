#![feature(try_trait)]
#![feature(try_from)]
#![feature(int_to_from_bytes)]
#![feature(alloc)]
#![feature(core_intrinsics)]
#![cfg_attr(not(test), no_std)]

#[cfg(test)]
#[macro_use]
extern crate std as alloc;

#[cfg(not(test))]
#[macro_use]
extern crate alloc;

#[cfg(test)]
#[macro_use]
extern crate core;

extern crate hashbrown;

#[macro_use]
mod decode;
mod embedder;
mod frame;
mod function;
mod global;
mod inst;
mod label;
mod memory;
mod module;
mod spectest;
mod stack;
mod store;
mod table;
mod trap;
mod validate;
mod value;
mod value_type;
mod vm;

pub use self::embedder::{decode_module, init_store, instantiate_module, validate_module};
pub use self::module::{ExternalModule, ExternalModules};
pub use self::spectest::create_spectest;
pub use self::trap::Trap;
pub use self::value::Values;
pub use self::vm::Vm;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;

    macro_rules! test_eval {
        ($fn_name:ident, $file_name:expr, $call_arguments: expr, $expect_value: expr) => {
            #[test]
            fn $fn_name() {
                let mut file = File::open(format!("./dist/{}.wasm", $file_name)).unwrap();
                let mut bytes = vec![];
                file.read_to_end(&mut bytes).unwrap();

                let store = init_store();
                let section = decode_module(&bytes);
                let mut vm = instantiate_module(store, section, Default::default()).unwrap();
                let actual = vm.run("_subject", $call_arguments);
                assert_eq!(actual, format!("i32:{}", $expect_value));
            }
        };
    }

    test_eval!(evaluate_cons8, "cons8", vec![], 42);
    test_eval!(
        evaluate_add_simple,
        "add",
        vec![Values::I32(3), Values::I32(4)],
        7
    );
    test_eval!(evaluate_sub, "sub", vec![Values::I32(10)], 90);
    test_eval!(
        evaluate_add_five,
        "add_five",
        vec![Values::I32(3), Values::I32(4)],
        17
    );
    test_eval!(evaluate_if_lt_1, "if_lt", vec![Values::I32(10)], 15);
    test_eval!(evaluate_if_lt_2, "if_lt", vec![Values::I32(9)], 19);
    test_eval!(evaluate_if_lt_3, "if_lt", vec![Values::I32(11)], 26);

    test_eval!(evaluate_if_gt_1, "if_gt", vec![Values::I32(10)], 15);
    test_eval!(evaluate_if_gt_2, "if_gt", vec![Values::I32(15)], 25);
    test_eval!(evaluate_if_gt_3, "if_gt", vec![Values::I32(5)], 20);

    test_eval!(evaluate_if_eq_1, "if_eq", vec![Values::I32(10)], 15);
    test_eval!(evaluate_if_eq_2, "if_eq", vec![Values::I32(11)], 21);
    test_eval!(evaluate_fib, "fib", vec![Values::I32(15)], 610);
    test_eval!(evaluate_5_count, "count", vec![Values::I32(5)], 35);
    test_eval!(evaluate_10_count, "count", vec![Values::I32(10)], 145);
    test_eval!(evaluate_100_count, "count", vec![Values::I32(100)], 14950);
}
