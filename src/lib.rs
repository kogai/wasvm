mod byte;
mod utils;

use byte::{FunctionInstance, Op, Values};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
struct Store {
    function_instances: HashMap<String, FunctionInstance>,
}

impl Store {
    fn call(&self, key: &str, arguments: Vec<Values>) -> Option<Vec<Op>> {
        self.function_instances.get(key).map(|f| f.call(arguments))
    }
}

#[derive(Debug, PartialEq)]
struct Frame {
    locals: Vec<Values>,
}

#[derive(Debug, PartialEq)]
enum StackEntry {
    // Op(Op),
    Value(Values),
    // Label,
    Frame(Frame),
}

#[derive(Debug)]
pub struct Vm {
    store: Store,
    stack: Vec<StackEntry>,
}

impl Vm {
    pub fn new(bytes: Vec<u8>) -> Self {
        let mut bytes = byte::Byte::new(bytes);
        let function_instances = bytes
            .decode()
            .expect("Instantiate function has been failured.");
        Vm {
            store: Store { function_instances },
            stack: vec![],
        }
    }

    fn pop(&mut self) -> Option<StackEntry> {
        unimplemented!();
    }

    pub fn run(&mut self, arguments: Vec<Values>) {
        match self.store.call("_subject", arguments) {
            Some(expressions) => {
                // for expression in expressions.iter() {
                //     self.stack.push(expression.to_owned());
                // }
                unimplemented!();
            }
            None => println!("'_subject' did not implemented."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use utils::read_wasm;

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

    // #[test]
    // fn it_can_evaluate_multiple_fns() {
    //     let wasm = read_wasm("./dist/multiple.wasm").unwrap();
    //     let mut vm = Vm::new(wasm);
    //     // assert_eq!(
    //     //     vm.store,
    //     //     Store {
    //     //         function_instances: HashMap::from_iter(
    //     //             vec![(
    //     //                 "_subject".to_owned(),
    //     //                 FunctionInstance {
    //     //                     function_type: FunctionType {
    //     //                         parameters: vec![],
    //     //                         returns: vec![byte::ValueTypes::I32],
    //     //                     },
    //     //                     locals: vec![],
    //     //                     type_idex: 0,
    //     //                     body: vec![Op::Const(42)],
    //     //                 }
    //     //             )].into_iter()
    //     //         )
    //     //     }
    //     // );
    // }

    #[test]
    fn it_can_evaluate_cons8() {
        let wasm = read_wasm("./dist/cons8.wasm").unwrap();
        let mut vm = Vm::new(wasm);
        vm.run(vec![]);
        assert_eq!(vm.pop(), Some(StackEntry::Value(Values::I32(42))));
    }

    #[test]
    fn it_can_evaluate_add() {
        let wasm = read_wasm("./dist/add.wasm").unwrap();
        let mut vm = Vm::new(wasm);
        vm.run(vec![]);
        assert_eq!(vm.pop(), Some(StackEntry::Value(Values::I32(42))));
    }
}
