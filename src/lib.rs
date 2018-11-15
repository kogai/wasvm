pub mod byte;
mod utils;
use byte::{FunctionInstance, Op, Values};
// use std::collections::LinkedList;
// use std::iter::FromIterator;

#[derive(Debug, PartialEq)]
struct Store {
    function_instances: Vec<FunctionInstance>,
}

impl Store {
    fn call(&self, fn_idx: usize) -> Option<&FunctionInstance> {
        self.function_instances.get(fn_idx)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Frame {
    locals: Vec<Values>,
    function_idx: usize,
    return_ptr: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum StackEntry {
    Empty,
    Value(Values),
    Label(Vec<Op>),
    Frame(Frame),
}

#[derive(Debug)]
pub struct Stack {
    entries: Vec<StackEntry>,
    stack_ptr: usize, // Start from 1
    frame_ptr: Vec<usize>,
    is_empty: bool,
}

impl Stack {
    fn new(stack_size: usize) -> Self {
        let entries = vec![StackEntry::Empty; stack_size];
        Stack {
            entries,
            stack_ptr: 0,
            frame_ptr: vec![],
            is_empty: false,
        }
    }

    fn increase(&mut self, count: usize) {
        self.stack_ptr += count;
    }

    fn get(&self, ptr: usize) -> Option<StackEntry> {
        self.entries.get(ptr).map(|e| e.to_owned())
    }

    fn set(&mut self, ptr: usize, entry: StackEntry) {
        self.entries[ptr] = entry;
    }

    fn push(&mut self, entry: StackEntry) {
        self.entries[self.stack_ptr] = entry;
        self.stack_ptr += 1;
    }

    // TODO: Return with ownership
    pub fn pop(&mut self) -> Option<&StackEntry> {
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
    pub stack: Stack,
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

    fn evaluate_instructions(&mut self, expressions: &Vec<Op>) -> Option<()> {
        for expression in expressions.iter() {
            match expression {
                Op::GetLocal(idx) => {
                    let frame_ptr = self.stack.frame_ptr.last()?.clone();
                    let value = self.stack.get(*idx + frame_ptr)?;
                    self.stack.push(value);
                }
                Op::SetLocal(idx) => {
                    let value = self.stack.pop().map(|s| s.to_owned())?;
                    let frame_ptr = self.stack.frame_ptr.last()?.clone();
                    self.stack.set(*idx + frame_ptr, value);
                }
                Op::Call(idx) => {
                    let operand = self.stack.pop_value().clone();
                    self.call(*idx, vec![operand]);
                    self.evaluate();
                }
                Op::I32Add => {
                    let right = self.stack.pop_value().clone();
                    let left = self.stack.pop_value().clone();
                    let result = StackEntry::Value(left + right);
                    self.stack.push(result);
                }
                Op::I32Sub => {
                    let right = self.stack.pop_value().clone();
                    let left = self.stack.pop_value().clone();
                    let result = StackEntry::Value(left - right);
                    self.stack.push(result);
                }
                Op::I32Const(n) => {
                    self.stack.push(StackEntry::Value(Values::I32(*n)));
                }
                Op::Select => {
                    let cond = &self.stack.pop_value().clone();
                    let false_br = self.stack.pop_value().clone();
                    let true_br = self.stack.pop_value().clone();
                    if cond.is_truthy() {
                        self.stack.push(StackEntry::Value(true_br));
                    } else {
                        self.stack.push(StackEntry::Value(false_br));
                    }
                }
                Op::LessThanSign | Op::LessThanUnsign => {
                    let right = &self.stack.pop_value().clone();
                    let left = &self.stack.pop_value().clone();
                    let cond = left.lt(right);
                    self.stack
                        .push(StackEntry::Value(Values::I32(if cond { 1 } else { 0 })));
                }
                Op::GreaterThanSign | Op::GreaterThanUnsign => {
                    let right = &self.stack.pop_value().clone();
                    let left = &self.stack.pop_value().clone();
                    let cond = left.gt(right);
                    self.stack
                        .push(StackEntry::Value(Values::I32(if cond { 1 } else { 0 })));
                }
                Op::Equal => {
                    let right = &self.stack.pop_value().clone();
                    let left = &self.stack.pop_value().clone();
                    let cond = left.eq(right);
                    self.stack
                        .push(StackEntry::Value(Values::I32(if cond { 1 } else { 0 })));
                }
                Op::NotEqual => {
                    let right = &self.stack.pop_value().clone();
                    let left = &self.stack.pop_value().clone();
                    let cond = left.neq(right);
                    self.stack
                        .push(StackEntry::Value(Values::I32(if cond { 1 } else { 0 })));
                }
                Op::If(if_ops, else_ops) => {
                    let cond = &self.stack.pop_value().clone();
                    let (_return_type, if_insts) = if_ops.split_first()?;
                    if cond.is_truthy() {
                        self.evaluate_instructions(&if_insts.to_vec());
                    } else {
                        if !else_ops.is_empty() {
                            self.evaluate_instructions(else_ops);
                        }
                    }
                }
                Op::I64ExtendUnsignI32
                | Op::I64Const(_)
                | Op::TeeLocal(_)
                | Op::I64Mul
                | Op::I64And
                | Op::I64ShiftRightUnsign
                | Op::I32Mul
                | Op::I32WrapI64
                | Op::Return
                | Op::TypeI32 => unreachable!(),
            };
        }
        Some(())
    }

    fn evaluate_frame(&mut self, instructions: &Vec<Op>) -> Option<()> {
        let _ = self.evaluate_instructions(instructions);
        let return_value = self.stack.pop_value().to_owned();
        let frame_ptr = self.stack.frame_ptr.pop().map(|f| f.to_owned())?;
        self.stack.stack_ptr = frame_ptr;
        self.stack.push(StackEntry::Value(return_value));
        Some(())
    }

    fn call(&mut self, function_idx: usize, arguments: Vec<Values>) {
        let frame = StackEntry::Frame(Frame {
            locals: arguments,
            return_ptr: self.stack.stack_ptr,
            function_idx,
        });
        self.stack.push(frame);
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
                    self.evaluate_frame(&expressions);
                }
                Some(StackEntry::Frame(frame)) => {
                    let _offset = frame.locals.len();
                    self.stack.frame_ptr.push(frame.return_ptr);
                    for local in frame.locals {
                        self.stack.push(StackEntry::Value(local));
                    }
                    let fn_instance = self.store.call(frame.function_idx);
                    let (expressions, locals) =
                        fn_instance.map(|f| f.call()).unwrap_or((vec![], vec![]));
                    let label = StackEntry::Label(expressions);
                    self.stack.increase(locals.len());
                    self.stack.push(label);
                }
                Some(StackEntry::Empty) | None => unreachable!("Invalid popping stack."),
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
    #[test]
    fn stack_ptr() {
        let mut stack = Stack::new(4);
        stack.push(StackEntry::Value(Values::I32(1)));
        stack.set(2, StackEntry::Value(Values::I32(2)));
        assert_eq!(stack.pop(), Some(&StackEntry::Value(Values::I32(1))));
        assert_eq!(stack.get(2), Some(StackEntry::Value(Values::I32(2))));
    }
    test_eval!(evaluate_cons8, "cons8", vec![], Values::I32(42));
    test_eval!(
        evaluate_add_simple,
        "add",
        vec![Values::I32(3), Values::I32(4)],
        Values::I32(7)
    );
    test_eval!(evaluate_sub, "sub", vec![Values::I32(10)], Values::I32(90));
    test_eval!(
        evaluate_add_five,
        "add_five",
        vec![Values::I32(3), Values::I32(4)],
        Values::I32(17)
    );
    test_eval!(
        evaluate_if_lt_1,
        "if_lt",
        vec![Values::I32(10)],
        Values::I32(15)
    );
    test_eval!(
        evaluate_if_lt_2,
        "if_lt",
        vec![Values::I32(9)],
        Values::I32(19)
    );
    test_eval!(
        evaluate_if_lt_3,
        "if_lt",
        vec![Values::I32(11)],
        Values::I32(26)
    );

    test_eval!(
        evaluate_if_gt_1,
        "if_gt",
        vec![Values::I32(10)],
        Values::I32(15)
    );
    test_eval!(
        evaluate_if_gt_2,
        "if_gt",
        vec![Values::I32(15)],
        Values::I32(25)
    );
    test_eval!(
        evaluate_if_gt_3,
        "if_gt",
        vec![Values::I32(5)],
        Values::I32(20)
    );

    test_eval!(
        evaluate_if_eq_1,
        "if_eq",
        vec![Values::I32(10)],
        Values::I32(15)
    );
    test_eval!(
        evaluate_if_eq_2,
        "if_eq",
        vec![Values::I32(11)],
        Values::I32(21)
    );
}
