#![feature(try_trait)]

pub mod byte;
mod code;
mod function;
mod inst;
mod memory;
mod store;
mod trap;
mod utils;
pub mod value;

use inst::Inst;
use std::rc::Rc;
use store::Store;
use trap::Result;
use value::Values;

#[derive(Debug, PartialEq, Clone)]
pub struct Frame {
    locals: Vec<Values>,
    function_idx: usize,
    return_ptr: usize,
}

#[derive(Debug, PartialEq)]
pub enum StackEntry {
    Empty,
    Value(Values),
    Label(Vec<Inst>),
    Frame(Frame),
}

#[derive(Debug)]
pub struct Stack {
    entries: Vec<Rc<StackEntry>>,
    stack_ptr: usize, // Start from 1
    frame_ptr: Vec<usize>,
    is_empty: bool,
}

impl Stack {
    fn new(stack_size: usize) -> Self {
        let entries = vec![Rc::new(StackEntry::Empty); stack_size];
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

    fn get(&self, ptr: usize) -> Option<Rc<StackEntry>> {
        self.entries.get(ptr).map(|rc| rc.clone())
    }

    fn set(&mut self, ptr: usize, entry: Rc<StackEntry>) {
        self.entries[ptr] = entry.clone();
    }

    fn push(&mut self, entry: Rc<StackEntry>) {
        self.entries[self.stack_ptr] = entry.clone();
        self.stack_ptr += 1;
    }

    pub fn pop(&mut self) -> Option<Rc<StackEntry>> {
        if self.stack_ptr == 0 {
            self.is_empty = true;
            None
        } else {
            self.stack_ptr -= 1;
            self.entries.get(self.stack_ptr).map(|rc| rc.clone())
        }
    }

    fn pop_value(&mut self) -> Values {
        let value = self.pop().expect("Expect to popp value but got None");
        match *value {
            StackEntry::Value(ref v) => v.to_owned(),
            ref x => unreachable!(format!("Expect to popp value but got {:?}", x).as_str()),
        }
    }
}

pub struct Vm {
    store: Store,
    pub stack: Stack,
}

impl Vm {
    pub fn new(bytes: Vec<u8>) -> Result<Self> {
        let mut bytes = byte::Byte::new(bytes);
        match bytes.decode() {
            Ok(store) => Ok(Vm {
                store,
                stack: Stack::new(65536),
            }),
            Err(err) => Err(err),
        }
    }

    // FIXME: Change to return Result<()>
    fn evaluate_instructions(&mut self, expressions: &Vec<Inst>) -> Option<()> {
        use self::Inst::*;
        let mut result = Some(());
        for expression in expressions.iter() {
            // println!("{:?}", &expression);
            match expression {
                GetLocal(idx) => {
                    let frame_ptr = self.stack.frame_ptr.last()?.clone();
                    let value = self.stack.get(*idx + frame_ptr)?;
                    self.stack.push(value);
                }
                SetLocal(idx) => {
                    let value = self.stack.pop().map(|s| s.to_owned())?;
                    let frame_ptr = self.stack.frame_ptr.last()?.clone();
                    self.stack.set(*idx + frame_ptr, value);
                }
                TeeLocal(idx) => {
                    let value = self.stack.pop().map(|s| s.to_owned())?;
                    self.stack.push(value.clone());
                    let frame_ptr = self.stack.frame_ptr.last()?.clone();
                    self.stack.set(*idx + frame_ptr, value);
                }
                Call(idx) => {
                    let operand = self.stack.pop_value();
                    self.call(*idx, vec![operand]);
                    let _ = self.evaluate();
                }
                I32Add | I64Add => {
                    let right = self.stack.pop_value();
                    let left = self.stack.pop_value();
                    let result = StackEntry::Value(left.add(&right));
                    self.stack.push(Rc::new(result));
                }
                I32Sub | I64Sub => {
                    let right = self.stack.pop_value();
                    let left = self.stack.pop_value();
                    let result = StackEntry::Value(left.sub(&right));
                    self.stack.push(Rc::new(result));
                }
                I32Mul | I64Mul => {
                    let right = self.stack.pop_value();
                    let left = self.stack.pop_value();
                    let result = StackEntry::Value(left.mul(&right));
                    self.stack.push(Rc::new(result));
                }
                I32DivUnsign | I64DivUnsign => {
                    let right = self.stack.pop_value();
                    let left = self.stack.pop_value();
                    match left.div_u(&right) {
                        Ok(result) => {
                            self.stack.push(Rc::new(StackEntry::Value(result)));
                        }
                        // FIXME: May handle trap properly.
                        Err(_trap) => {
                            result = None;
                            break;
                        }
                    }
                }
                I32DivSign | I64DivSign => {
                    let right = self.stack.pop_value();
                    let left = self.stack.pop_value();
                    match left.div_s(&right) {
                        Ok(result) => {
                            self.stack.push(Rc::new(StackEntry::Value(result)));
                        }
                        // FIXME: May handle trap properly.
                        Err(_trap) => {
                            result = None;
                            break;
                        }
                    }
                }
                I32RemSign | I64RemSign => {
                    let right = self.stack.pop_value();
                    let left = self.stack.pop_value();
                    match left.rem_s(&right) {
                        Ok(result) => {
                            self.stack.push(Rc::new(StackEntry::Value(result)));
                        }
                        // FIXME: May handle trap properly.
                        Err(_trap) => {
                            result = None;
                            break;
                        }
                    }
                }
                I32RemUnsign | I64RemUnsign => {
                    let right = self.stack.pop_value();
                    let left = self.stack.pop_value();
                    match left.rem_u(&right) {
                        Ok(result) => {
                            self.stack.push(Rc::new(StackEntry::Value(result)));
                        }
                        // FIXME: May handle trap properly.
                        Err(_trap) => {
                            result = None;
                            break;
                        }
                    }
                }
                I32Const(n) => {
                    self.stack.push(Rc::new(StackEntry::Value(Values::I32(*n))));
                }
                I64Const(n) => {
                    self.stack.push(Rc::new(StackEntry::Value(Values::I64(*n))));
                }
                Select => {
                    let cond = &self.stack.pop_value();
                    let false_br = self.stack.pop_value();
                    let true_br = self.stack.pop_value();
                    if cond.is_truthy() {
                        self.stack.push(Rc::new(StackEntry::Value(true_br)));
                    } else {
                        self.stack.push(Rc::new(StackEntry::Value(false_br)));
                    }
                }
                LessThanSign | I64LessThanSign => {
                    let right = &self.stack.pop_value();
                    let left = &self.stack.pop_value();
                    self.stack
                        .push(Rc::new(StackEntry::Value(left.less_than(right))));
                }
                LessThanUnsign | I64LessThanUnSign => {
                    let right = &self.stack.pop_value();
                    let left = &self.stack.pop_value();
                    self.stack
                        .push(Rc::new(StackEntry::Value(left.less_than_unsign(right))));
                }
                I32LessEqualSign | I64LessEqualSign => {
                    let right = &self.stack.pop_value();
                    let left = &self.stack.pop_value();
                    self.stack
                        .push(Rc::new(StackEntry::Value(left.less_than_equal(right))));
                }
                I32LessEqualUnsign | I64LessEqualUnSign => {
                    let right = &self.stack.pop_value();
                    let left = &self.stack.pop_value();
                    self.stack.push(Rc::new(StackEntry::Value(
                        left.less_than_equal_unsign(right),
                    )));
                }
                I32GreaterEqualSign | I64GreaterEqualSign => {
                    let right = &self.stack.pop_value();
                    let left = &self.stack.pop_value();
                    self.stack
                        .push(Rc::new(StackEntry::Value(left.greater_than_equal(right))));
                }
                I32GreaterThanSign | I64GreaterThanSign => {
                    let right = &self.stack.pop_value();
                    let left = &self.stack.pop_value();
                    self.stack
                        .push(Rc::new(StackEntry::Value(left.greater_than(right))));
                }
                I32GreaterThanUnsign | I64GreaterThanUnSign => {
                    let right = &self.stack.pop_value();
                    let left = &self.stack.pop_value();
                    self.stack
                        .push(Rc::new(StackEntry::Value(left.greater_than_unsign(right))));
                }
                I32GreaterEqualUnsign | I64GreaterEqualUnSign => {
                    let right = &self.stack.pop_value();
                    let left = &self.stack.pop_value();
                    self.stack.push(Rc::new(StackEntry::Value(
                        left.greater_than_equal_unsign(right),
                    )));
                }
                Equal | I64Equal => {
                    let right = &self.stack.pop_value();
                    let left = &self.stack.pop_value();
                    self.stack
                        .push(Rc::new(StackEntry::Value(left.equal(right))));
                }
                NotEqual | I64NotEqual => {
                    let right = &self.stack.pop_value();
                    let left = &self.stack.pop_value();
                    self.stack
                        .push(Rc::new(StackEntry::Value(left.not_equal(right))));
                }
                I32Or | I64Or => {
                    let right = &self.stack.pop_value();
                    let left = &self.stack.pop_value();
                    let result = left.or(right);
                    self.stack.push(Rc::new(StackEntry::Value(result)));
                }
                I32Xor | I64Xor => {
                    let right = &self.stack.pop_value();
                    let left = &self.stack.pop_value();
                    let result = left.xor(right);
                    self.stack.push(Rc::new(StackEntry::Value(result)));
                }
                I32And | I64And => {
                    let right = &self.stack.pop_value();
                    let left = &self.stack.pop_value();
                    let result = left.and(right);
                    self.stack.push(Rc::new(StackEntry::Value(result)));
                }
                If(_return_type, if_ops, else_ops) => {
                    let cond = &self.stack.pop_value();
                    if cond.is_truthy() {
                        self.evaluate_instructions(if_ops);
                    } else {
                        if !else_ops.is_empty() {
                            self.evaluate_instructions(else_ops);
                        }
                    }
                }
                Return => {
                    unimplemented!();
                }
                I64ExtendUnsignI32 => {
                    let value = &self.stack.pop_value();
                    let result = value.extend_to_i64();
                    self.stack.push(Rc::new(StackEntry::Value(result)));
                }
                I32ShiftLeft | I64ShiftLeft => {
                    let i2 = &self.stack.pop_value();
                    let i1 = &self.stack.pop_value();
                    let result = i1.shift_left(i2);
                    self.stack.push(Rc::new(StackEntry::Value(result)));
                }
                I32ShiftRIghtSign | I64ShiftRightSign => {
                    let i2 = &self.stack.pop_value();
                    let i1 = &self.stack.pop_value();
                    let result = i1.shift_right_sign(i2);
                    self.stack.push(Rc::new(StackEntry::Value(result)));
                }
                I32ShiftRightUnsign | I64ShiftRightUnsign => {
                    let i2 = &self.stack.pop_value();
                    let i1 = &self.stack.pop_value();
                    let result = i1.shift_right_unsign(i2);
                    self.stack.push(Rc::new(StackEntry::Value(result)));
                }
                I32WrapI64 => {
                    let i = &self.stack.pop_value();
                    if let Values::I64(n) = i {
                        let result = (*n % 2_i64.pow(32)) as i32;
                        self.stack
                            .push(Rc::new(StackEntry::Value(Values::I32(result))));
                    } else {
                        unreachable!();
                    }
                }
                I32RotateLeft | I64RotateLeft => {
                    let i2 = &self.stack.pop_value();
                    let i1 = &self.stack.pop_value();
                    let result = i1.wasm_rotate_left(i2);
                    self.stack.push(Rc::new(StackEntry::Value(result)));
                }
                I32RotateRight | I64RotateRight => {
                    let i2 = &self.stack.pop_value();
                    let i1 = &self.stack.pop_value();
                    let result = i1.wasm_rotate_right(i2);
                    self.stack.push(Rc::new(StackEntry::Value(result)));
                }
                I32CountLeadingZero | I64CountLeadingZero => {
                    let l = &self.stack.pop_value();
                    let result = l.count_leading_zero();
                    self.stack.push(Rc::new(StackEntry::Value(result)));
                }
                I32CountTrailingZero | I64CountTrailingZero => {
                    let l = &self.stack.pop_value();
                    let result = l.count_trailing_zero();
                    self.stack.push(Rc::new(StackEntry::Value(result)));
                }
                I32CountNonZero | I64CountNonZero => {
                    let l = &self.stack.pop_value();
                    let result = l.pop_count();
                    self.stack.push(Rc::new(StackEntry::Value(result)));
                }
                TypeEmpty => unreachable!(),
                TypeI32 => unreachable!(),
                I32Load8Unsign(_align, offset) => {
                    let i = match self.stack.pop_value() {
                        Values::I32(i) => i,
                        x => unreachable!("{:?}", x),
                    } as u32;
                    let ea = i + *offset;
                    if (ea + 4/* = 32 / 4 */) > 999
                    /* mem.data */
                    {
                        // FIXME:
                        return None;
                        // return Err(Trap::MemoryAccessOutOfBounds)
                    };
                    let n = (i as u64) % 2u64.pow(32);
                    // Get 4 byte from target address
                    // data = [ea..32/8];
                    // c = extend_to_i32(data)
                    // push(c)
                    // unimplemented!();
                    self.stack
                        .push(Rc::new(StackEntry::Value(Values::I32(0x61))));
                }
                I32Load(align, offset)
                | I64Load(align, offset)
                | F32Load(align, offset)
                | F64Load(align, offset)
                | I32Load8Sign(align, offset)
                | I32Load16Sign(align, offset)
                | I32Load16Unsign(align, offset)
                | I64Load8Sign(align, offset)
                | I64Load8Unsign(align, offset)
                | I64Load16Sign(align, offset)
                | I64Load16Unsign(align, offset)
                | I64Load32Sign(align, offset)
                | I64Load32Unsign(align, offset)
                | I32Store(align, offset)
                | I64Store(align, offset)
                | F32Store(align, offset)
                | F64Store(align, offset)
                | I32Store8(align, offset)
                | I32Store16(align, offset)
                | I64Store8(align, offset)
                | I64Store16(align, offset)
                | I64Store32(align, offset) => {
                    unimplemented!();
                }
                I32EqualZero | I64EqualZero => {
                    let l = &self.stack.pop_value();
                    let result = l.equal_zero();
                    self.stack.push(Rc::new(StackEntry::Value(result)));
                }
            };
            // println!("[{}] {:?}", self.stack.stack_ptr, self.stack.entries);
            // println!("");
        }
        result
    }

    fn evaluate_frame(&mut self, instructions: &Vec<Inst>) -> Option<()> {
        self.evaluate_instructions(instructions)?;
        let return_value = self.stack.pop_value();
        let frame_ptr = self.stack.frame_ptr.pop()?;
        self.stack.stack_ptr = frame_ptr;
        self.stack.push(Rc::new(StackEntry::Value(return_value)));
        Some(())
    }

    fn call(&mut self, function_idx: usize, arguments: Vec<Values>) {
        let frame = StackEntry::Frame(Frame {
            locals: arguments,
            return_ptr: self.stack.stack_ptr,
            function_idx,
        });
        self.stack.push(Rc::new(frame));
    }

    fn evaluate(&mut self) -> Result<()> {
        let mut result = None;
        while !self.stack.is_empty {
            let popped = self.stack.pop().expect("Invalid popping stack.");
            match *popped {
                StackEntry::Value(ref v) => {
                    result = Some(StackEntry::Value(v.to_owned()));
                    break;
                }
                StackEntry::Label(ref expressions) => {
                    self.evaluate_frame(&expressions)?;
                }
                StackEntry::Frame(ref frame) => {
                    let _offset = frame.locals.len();
                    self.stack.frame_ptr.push(frame.return_ptr);
                    for local in frame.clone().locals {
                        self.stack.push(Rc::new(StackEntry::Value(local)));
                    }
                    let fn_instance = self.store.call(frame.function_idx);
                    let (expressions, locals) =
                        fn_instance.map(|f| f.call()).unwrap_or((vec![], vec![]));
                    let label = StackEntry::Label(expressions);
                    self.stack.increase(locals.len());
                    self.stack.push(Rc::new(label));
                }
                StackEntry::Empty => unreachable!("Invalid popping stack."),
            }
        }
        self.stack.push(Rc::new(
            result.expect("Call stack may return with null value"),
        ));
        Ok(())
    }

    pub fn run(&mut self, invoke: &str, arguments: Vec<Values>) -> String {
        let start_idx = self.store.get_function_idx(invoke);
        self.call(start_idx, arguments);
        match self.evaluate() {
            Ok(_) => match self.stack.pop_value() {
                Values::I32(v) => format!("{}", v),
                Values::I64(v) => format!("{}", v),
            },
            Err(err) => String::from(err),
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
                let mut vm = Vm::new(wasm).unwrap();
                let actual = vm.run("_subject", $call_arguments);
                assert_eq!(actual, format!("{}", $expect_value));
            }
        };
    }
    #[test]
    fn stack_ptr() {
        let mut stack = Stack::new(4);
        stack.push(Rc::new(StackEntry::Value(Values::I32(1))));
        stack.set(2, Rc::new(StackEntry::Value(Values::I32(2))));
        assert_eq!(*stack.pop().unwrap(), StackEntry::Value(Values::I32(1)));
        assert_eq!(*stack.get(2).unwrap(), StackEntry::Value(Values::I32(2)));
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
