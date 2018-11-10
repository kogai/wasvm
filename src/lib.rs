mod byte;
mod utils;

use byte::{FunctionInstance, Op, Values};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
struct Store {
    function_instances: HashMap<String, FunctionInstance>,
}

impl Store {
    fn call(&self, key: &str) -> Option<Vec<Op>> {
        self.function_instances.get(key).map(|f| f.call())
    }
}

#[derive(Debug, PartialEq)]
struct Frame {
    locals: Vec<Values>,
    return_ptr: usize,
}

#[derive(Debug, PartialEq)]
enum StackEntry {
    // Op(Op),
    Value(Values),
    // Label,
    Frame(Frame),
}

#[derive(Debug)]
struct Stack {
    entries: Vec<StackEntry>,
    stack_ptr: usize,
}

impl Stack {
    fn new(stack_size: usize) -> Self {
        let mut entries = Vec::with_capacity(stack_size);
        unsafe {
            entries.set_len(stack_size);
        };
        Stack {
            entries,
            stack_ptr: 0,
        }
    }

    fn push(&mut self, entry: StackEntry) {
        self.entries[self.stack_ptr] = entry;
        self.stack_ptr += 1;
    }

    fn pop(&mut self) -> Option<&StackEntry> {
        self.stack_ptr -= 1;
        self.entries.get(self.stack_ptr)
    }
}

#[derive(Debug)]
pub struct Vm {
    store: Store,
    stack: Stack,
}

impl Vm {
    pub fn new(bytes: Vec<u8>) -> Self {
        let mut bytes = byte::Byte::new(bytes);
        let function_instances = bytes
            .decode()
            .expect("Instantiate function has been failured.");
        Vm {
            store: Store { function_instances },
            stack: Stack::new(2048),
        }
    }

    pub fn run(&mut self, arguments: Vec<Values>) {
        // let frame = Frame {
        //     locals: Vec<Values>,
        //     return_ptr: usize,
        // };
        match self.store.call("_subject") {
            Some(expressions) => {
                for expression in expressions.iter() {
                    match expression {
                        Op::GetLocal(idx) => self.stack.push(StackEntry::Value(
                            arguments
                                .get(*idx)
                                .map(|v| v.to_owned())
                                .expect(format!("GetLocal({}) has been failured.", idx).as_str()),
                        )),
                        Op::Add => {
                            let left = match self
                                .stack
                                .pop()
                                .expect(format!("Left-operand does not exists.").as_str())
                            {
                                StackEntry::Value(Values::I32(l)) => *l,
                                _ => unimplemented!(),
                            };
                            let right = match self
                                .stack
                                .pop()
                                .expect(format!("Right-operand does not exists.").as_str())
                            {
                                StackEntry::Value(Values::I32(l)) => *l,
                                _ => unimplemented!(),
                            };
                            self.stack
                                .push(StackEntry::Value(Values::I32(left + right)));
                        }
                        Op::Const(n) => self.stack.push(StackEntry::Value(Values::I32(*n))),
                    };
                }
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
        assert_eq!(vm.stack.pop(), Some(&StackEntry::Value(Values::I32(42))));
    }

    #[test]
    fn it_can_evaluate_add() {
        let wasm = read_wasm("./dist/add.wasm").unwrap();
        let mut vm = Vm::new(wasm);
        vm.run(vec![Values::I32(3), Values::I32(4)]);
        assert_eq!(vm.stack.pop(), Some(&StackEntry::Value(Values::I32(7))));
    }
}
