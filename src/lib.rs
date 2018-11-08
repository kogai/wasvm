use std::collections::HashMap;
mod byte;
mod utils;

#[derive(Debug, PartialEq)]
enum Op {
    Const(i32),
}

#[derive(Debug, PartialEq)]
struct FunctionType {
    parameters: Vec<byte::ValueTypes>,
    returns: Vec<byte::ValueTypes>,
}

#[derive(Debug, PartialEq)]
struct FunctionInstance {
    function_type: FunctionType,
    locals: Vec<byte::Values>,
    type_idex: u32,
    body: Vec<Op>,
}

#[derive(Debug, PartialEq)]
struct Store {
    function_instances: HashMap<u32, FunctionInstance>,
}

impl Store {
    fn new(bytecode: Vec<byte::Code>) -> Self {
        Store {
            function_instances: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct Vm {
    store: Store,
    stack: Vec<Op>,
}

impl Vm {
    pub fn new(bytes: Vec<u8>) -> Self {
        let mut bytes = byte::Byte::new(bytes);
        let _ = bytes.decode();
        Vm {
            store: Store::new(bytes.bytes_decoded),
            stack: vec![],
        }
    }

    pub fn run(&mut self) {
        // match self.stack.pop_front() {
        //     Some(v) => v,
        //     _ => Value::I32(0),
        // }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::FromIterator;
    use utils::read_wasm;

    #[test]
    fn it_can_organize_functions() {
        let wasm = read_wasm("./dist/constant.wasm").unwrap();
        let vm = Vm::new(wasm);
        assert_eq!(
            vm.store,
            Store {
                function_instances: HashMap::from_iter(
                    vec![(
                        0,
                        FunctionInstance {
                            function_type: FunctionType {
                                parameters: vec![],
                                returns: vec![byte::ValueTypes::I32],
                            },
                            locals: vec![],
                            type_idex: 0,
                            body: vec![Op::Const(42)],
                        }
                    )].into_iter()
                )
            }
        );
    }

    //     #[test]
    //     fn it_can_push_constant() {
    //         let wasm = read_wasm("./dist/constant.wasm").unwrap();
    //         let mut vm = Vm::new(wasm);
    //         vm.decode();
    //         assert_eq!(vm.stack.pop_front(), Some(Value::I32(42)));
    //     }

    //     #[test]
    //     fn it_can_organize_modules() {
    //         let wasm = read_wasm("./dist/constant.wasm").unwrap();
    //         let mut vm = Vm::new(wasm);
    //         vm.decode();
    //         assert_eq!(
    //             vm.module,
    //             Module {
    //                 types: vec![(vec![], vec![ValueType::I32])],
    //                 func_addresses: vec![0]
    //             }
    //         );
    //     }

    //     #[test]
    //     fn it_can_evaluate_constant() {
    //         let wasm = read_wasm("./dist/constant.wasm").unwrap();
    //         let mut vm = Vm::new(wasm);
    //         vm.decode();
    //         assert_eq!(vm.run(), Value::I32(42));
    //     }
}
