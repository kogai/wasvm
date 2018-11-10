mod byte;
mod utils;
use byte::{FunctionInstance, Op, Values};

#[derive(Debug, PartialEq)]
struct Store {
    function_instances: Vec<FunctionInstance>,
    current_frame: Option<Frame>,
}

impl Store {
    fn call(&self, fn_idx: usize) -> Option<&FunctionInstance> {
        self.function_instances.get(fn_idx)
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Frame {
    locals: Vec<Values>,
    return_ptr: usize,
}

#[derive(Debug, PartialEq, Clone)]
enum StackEntry {
    Value(Values),
    Label(Vec<Op>),
    Frame(Frame),
}

#[derive(Debug)]
struct Stack {
    entries: Vec<StackEntry>,
    stack_ptr: usize,
    is_empty: bool,
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
            is_empty: false,
        }
    }

    fn push(&mut self, entry: StackEntry) {
        self.entries[self.stack_ptr] = entry;
        self.stack_ptr += 1;
    }

    fn pop(&mut self) -> Option<&StackEntry> {
        if self.stack_ptr == 0 {
            self.is_empty = true;
            None
        } else {
            self.stack_ptr -= 1;
            self.entries.get(self.stack_ptr)
        }
    }

    fn pop_value(&mut self) -> &Values {
        match self.pop() {
            Some(StackEntry::Value(v)) => v,
            x => unreachable!(format!("Expect to popp value but got {:?}", x).as_str()),
        }
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
            store: Store {
                function_instances,
                current_frame: None,
            },
            stack: Stack::new(2048),
        }
    }

    fn call(&mut self, fn_idx: usize, arguments: Vec<Values>) {
        let frame = StackEntry::Frame(Frame {
            locals: arguments,
            return_ptr: self.stack.stack_ptr,
        });
        let fn_instance = self.store.call(fn_idx);
        let expressions = fn_instance.map(|f| f.call()).unwrap_or(vec![]);
        let label = StackEntry::Label(expressions);

        self.stack.push(label);
        self.stack.push(frame);
    }

    fn evaluate_instructions(&mut self, expressions: Vec<Op>) -> Option<()> {
        let current_frame = self.store.current_frame.clone()?;
        for expression in expressions.iter() {
            match expression {
                Op::GetLocal(idx) => {
                    let argument = current_frame
                        .locals
                        .get(*idx)
                        .map(|v| v.to_owned())
                        .expect(format!("GetLocal({}) has been failured.", idx).as_str());
                    self.stack.push(StackEntry::Value(argument));
                }
                Op::SetLocal(idx) => {
                    unimplemented!();
                }
                Op::Call(idx) => {
                    let operand = self
                        .stack
                        .pop()
                        .expect(format!("Left-operand does not exists.").as_str());
                    let function = self.store.function_instances.get(*idx);
                    // let frame = StackEntry::Frame(Frame {
                    //     locals: arguments,
                    //     return_ptr: self.stack.stack_ptr,
                    // });
                    println!("{:?}", function);
                    unimplemented!();
                }
                Op::Add => {
                    let left = self.stack.pop_value().clone();
                    let right = self.stack.pop_value().clone();
                    let result = StackEntry::Value(left + right);
                    self.stack.push(result);
                }
                Op::Const(n) => {
                    self.stack.push(StackEntry::Value(Values::I32(*n)));
                }
            };
        }
        Some(())
    }

    fn evaluate(&mut self) {
        let mut result = None;
        while !self.stack.is_empty {
            let popped = self.stack.pop().map(|v| v.to_owned());
            match popped {
                Some(StackEntry::Value(v)) => {
                    result = Some(StackEntry::Value(v));
                    break;
                }
                Some(StackEntry::Label(expressions)) => {
                    self.evaluate_instructions(expressions);
                }
                Some(StackEntry::Frame(frame)) => {
                    self.store.current_frame = Some(frame.to_owned());
                }
                None => unreachable!("Invalid popping stack."),
            }
        }
        self.stack
            .push(result.expect("Call stack may return with null value"));
    }

    pub fn run(&mut self, arguments: Vec<Values>) {
        let start_idx = self
            .store
            .function_instances
            .iter()
            .position(|f| f.find("_subject"))
            .expect("Main function did not found.");
        self.call(start_idx, arguments);
        self.evaluate();
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
