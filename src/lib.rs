mod byte;
mod utils;
use byte::{FunctionInstance, Op, Values};

#[derive(Debug, PartialEq)]
struct Store {
    function_instances: Vec<FunctionInstance>,
}

impl Store {
    fn call(&self, key: &str) -> Option<Vec<Op>> {
        let xxx = self
            .function_instances
            .iter()
            .find(|f| f.find(key))
            .map(|f| f.call());
        xxx
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
                        Op::GetLocal(idx) => {
                            let argument = arguments
                                .get(*idx)
                                .map(|v| v.to_owned())
                                .expect(format!("GetLocal({}) has been failured.", idx).as_str());
                            self.stack.push(StackEntry::Value(argument));
                        }
                        Op::SetLocal(idx) => {
                            unimplemented!();
                        }
                        Op::Call(idx) => {
                            unimplemented!();
                        }
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

    macro_rules! test_eval {
        ($fn_name:ident, $file_name:expr, $call_arguments: expr, $expect_value: expr) => {
            #[test]
            fn $fn_name() {
                let wasm = read_wasm(format!("./dist/{}.wasm", $file_name)).unwrap();
                let mut vm = Vm::new(wasm);
                vm.run($call_arguments);
                assert_eq!(vm.stack.pop(), Some(&StackEntry::Value($expect_value)));
            }
        };
    }

    test_eval!(evaluate_cons8, "cons8", vec![], Values::I32(42));
    test_eval!(
        evaluate_add,
        "add",
        vec![Values::I32(3), Values::I32(4)],
        Values::I32(7)
    );
    test_eval!(
        evaluate_add_five,
        "add_five",
        vec![Values::I32(3), Values::I32(4)],
        Values::I32(17)
    );

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
}
