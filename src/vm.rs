use decode::Byte;
use function::FunctionType;
use inst::{Inst, Instructions};
use stack::Frame;
use stack::{Stack, StackEntry};
use store::Store;
use trap::{Result, Trap};
use value::Values;
use value_type::ValueTypes;

macro_rules! impl_load_inst {
    ($load_data_width: expr, $self: ident, $offset: ident, $value_kind: expr) => {{
        let width = $load_data_width / 8;
        let i = $self.stack.pop_value_ext_i32() as u32;
        let (effective_address, overflowed) = i.overflowing_add($offset);
        if overflowed {
            return Err(Trap::MemoryAccessOutOfBounds);
        };
        let (ptr, overflowed) = effective_address.overflowing_add(width);
        if overflowed || $self.store.data_size_small_than(ptr) {
            return Err(Trap::MemoryAccessOutOfBounds);
        };
        let data = $self.store.load_data(effective_address, ptr, &$value_kind);
        $self.stack.push(StackEntry::new_value(data))?;
    }};
}

macro_rules! impl_store_inst {
    ($data_width: expr, $self: ident, $offset: ident) => {{
        let c = $self.stack.pop_value_ext();
        let width = $data_width / 8;
        let i = $self.stack.pop_value_ext_i32() as u32;
        let (effective_address, overflowed) = i.overflowing_add($offset);
        if overflowed {
            return Err(Trap::MemoryAccessOutOfBounds);
        };
        let (ptr, overflowed) = effective_address.overflowing_add(width);
        if overflowed || $self.store.data_size_small_than(ptr) {
            return Err(Trap::MemoryAccessOutOfBounds);
        };
        $self.store.store_data(effective_address, ptr, c);
    }};
}

macro_rules! impl_unary_inst {
    ($self: ident, $op: ident) => {{
        let popped = $self.stack.pop_value_ext();
        let value = popped.$op();
        $self.stack.push(StackEntry::new_value(value))?;
    }};
}

macro_rules! impl_try_unary_inst {
    ($self: ident, $op: ident) => {{
        let popped = $self.stack.pop_value_ext();
        let value = popped.$op();
        match value {
            Ok(result) => {
                $self.stack.push(StackEntry::new_value(result))?;
            }
            Err(trap) => {
                return Err(trap);
            }
        }
    }};
}

macro_rules! impl_binary_inst {
    ($self: ident, $op: ident) => {{
        let right = $self.stack.pop_value_ext();
        let left = $self.stack.pop_value_ext();
        let value = left.$op(&right);
        $self.stack.push(StackEntry::new_value(value))?;
    }};
}

macro_rules! impl_try_binary_inst {
    ($self: ident, $op: ident) => {{
        let right = $self.stack.pop_value_ext();
        let left = $self.stack.pop_value_ext();
        let value = left.$op(&right);
        match value {
            Ok(result) => {
                $self.stack.push(StackEntry::new_value(result))?;
            }
            Err(trap) => {
                return Err(trap);
            }
        }
    }};
}

pub struct Vm {
    store: Store,
    stack: Stack,
}

impl Vm {
    pub fn new(bytes: Vec<u8>) -> Result<Self> {
        let mut bytes = Byte::new_with_drop(bytes);
        match bytes.decode() {
            Ok(store) => Ok(Vm {
                store,
                stack: Stack::new(65536),
            }),
            Err(err) => Err(err),
        }
    }

    fn evaluate_instructions(&mut self, instructions: &mut Instructions) -> Result<()> {
        use self::Inst::*;
        while !instructions.is_next_end_or_else() {
            let expression = instructions.pop().unwrap();
            match expression {
                Unreachable => {
                    unimplemented!("{:?}", expression);
                }
                Return => {
                    return Ok(());
                }
                Nop => {}
                Block(size) => {
                    let start_of_control = instructions.ptr - 1;
                    let continuation = start_of_control + size;
                    instructions.push_label(continuation);
                    let _block_type = instructions.pop().unwrap();
                    self.evaluate_instructions(instructions)?;
                    if continuation > instructions.ptr {
                        instructions.pop_label()?; // Drop own label.
                        instructions.ptr = continuation;
                    } else {
                        break;
                    }
                }
                Loop => {
                    let start_of_control = instructions.ptr - 1;
                    instructions.push_label(start_of_control);
                    let _block_type = instructions.pop().unwrap();
                    self.evaluate_instructions(instructions)?;
                    instructions.pop_label()?; // Drop own label.
                    instructions.pop()?; // Drop End instruction.
                }
                If(if_size, else_size) => {
                    let cond = &self.stack.pop_value_ext();
                    let start_of_if = instructions.ptr - 1;
                    let continuation = start_of_if + if_size + else_size;
                    let start_of_else = start_of_if + if_size;
                    instructions.push_label(continuation);
                    let _return_type = instructions.pop().unwrap();

                    if cond.is_truthy() {
                        self.evaluate_instructions(instructions)?;
                    } else {
                        instructions.jump_to(start_of_else);
                        if else_size > 0 {
                            self.evaluate_instructions(instructions)?;
                        }
                    }
                    instructions.jump_to_label(0);
                }
                Else => unreachable!(),
                End => break,
                Br(l) => {
                    instructions.jump_to_label(l);
                }
                BrIf(l) => {
                    let cond = &self.stack.pop_value_ext();
                    if cond.is_truthy() {
                        instructions.jump_to_label(l);
                    };
                }
                BrTable(ref tables, ref idx) => {
                    let i = if let Values::I32(i) = self.stack.pop_value_ext() {
                        i as usize
                    } else {
                        unreachable!();
                    };
                    let label = if i < tables.len() {
                        tables.get(i)?
                    } else {
                        idx
                    };
                    instructions.jump_to_label(*label);
                }
                Call(idx) => {
                    let count_of_arguments = self.store.call(idx)?.get_arity();
                    let mut arguments = vec![];
                    for _ in 0..count_of_arguments {
                        arguments.push(self.stack.pop_value_ext());
                    }
                    self.expand_frame(idx, arguments)?;
                    self.evaluate()?;
                }
                CallIndirect(_idx) => {
                    let ta = instructions.get_table_address();
                    let table = self.store.get_table_at(ta)?.clone();
                    let i = self.stack.pop_value_ext_i32() as u32;
                    if i >= table.len() {
                        return Err(Trap::MemoryAccessOutOfBounds);
                    }
                    let address = table.get_function_address(i)?;
                    // FIXME: Use idx pass from CallIndirect instruction :thinking:
                    let function_type = instructions.get_type_at(address)?;
                    let mut arguments = vec![];
                    for _ in 0..function_type.get_arity() {
                        arguments.push(self.stack.pop_value_ext());
                    }
                    arguments.reverse();

                    self.expand_frame(address as usize, arguments)?;
                    self.evaluate()?;
                }
                GetLocal(idx) => {
                    let frame_ptr = self.stack.get_frame_ptr();
                    let value = self.stack.get((idx as usize) + frame_ptr)?;
                    self.stack.push(value)?;
                }
                SetLocal(idx) => {
                    let value = self.stack.pop().map(|s| s.to_owned())?;
                    let frame_ptr = self.stack.get_frame_ptr();
                    self.stack.set((idx as usize) + frame_ptr, value)?;
                }
                TeeLocal(idx) => {
                    let value = self.stack.pop().map(|s| s.to_owned())?;
                    self.stack.push(value.clone())?;
                    let frame_ptr = self.stack.get_frame_ptr();
                    self.stack.set((idx as usize) + frame_ptr, value)?;
                }
                GetGlobal(idx) => {
                    let value = self.store.get_global(idx)?.to_owned();
                    self.stack.push(StackEntry::new_value(value))?;
                }
                SetGlobal(idx) => {
                    let value = self.stack.pop_value_ext();
                    self.store.set_global(idx, value);
                }
                I32Const(n) => self.stack.push(StackEntry::new_value(Values::I32(n)))?,
                I64Const(n) => self.stack.push(StackEntry::new_value(Values::I64(n)))?,
                F32Const(n) => self.stack.push(StackEntry::new_value(Values::F32(n)))?,
                F64Const(n) => self.stack.push(StackEntry::new_value(Values::F64(n)))?,
                I32Add | I64Add | F32Add | F64Add => impl_binary_inst!(self, add),
                I32Sub | I64Sub | F32Sub | F64Sub => impl_binary_inst!(self, sub),
                I32Mul | I64Mul | F32Mul | F64Mul => impl_binary_inst!(self, mul),
                I32DivUnsign | I64DivUnsign => impl_try_binary_inst!(self, div_u),
                I32DivSign | I64DivSign => impl_try_binary_inst!(self, div_s),
                F32Div | F64Div => impl_binary_inst!(self, div_f),
                I32RemSign | I64RemSign => impl_try_binary_inst!(self, rem_s),
                I32RemUnsign | I64RemUnsign => impl_try_binary_inst!(self, rem_u),
                F32Min | F64Min => impl_binary_inst!(self, min),
                F32Max | F64Max => impl_binary_inst!(self, max),
                F32Sqrt | F64Sqrt => impl_unary_inst!(self, sqrt),
                F32Ceil | F64Ceil => impl_unary_inst!(self, ceil),
                F32Floor | F64Floor => impl_unary_inst!(self, floor),
                F32Trunc | F64Trunc => impl_unary_inst!(self, trunc),
                F32Nearest | F64Nearest => impl_unary_inst!(self, nearest),
                Select => {
                    let cond = &self.stack.pop_value_ext();
                    let false_br = self.stack.pop_value_ext();
                    let true_br = self.stack.pop_value_ext();
                    if cond.is_truthy() {
                        self.stack.push(StackEntry::new_value(true_br))?;
                    } else {
                        self.stack.push(StackEntry::new_value(false_br))?;
                    }
                }
                DropInst => {
                    self.stack.pop_value_ext();
                }
                LessThanSign | I64LessThanSign | F32LessThan | F64LessThan => {
                    impl_binary_inst!(self, less_than)
                }
                LessThanUnsign | I64LessThanUnSign => impl_binary_inst!(self, less_than_unsign),
                I32LessEqualSign | I64LessEqualSign | F32LessEqual | F64LessEqual => {
                    impl_binary_inst!(self, less_than_equal)
                }
                I32LessEqualUnsign | I64LessEqualUnSign => {
                    impl_binary_inst!(self, less_than_equal_unsign)
                }
                I32GreaterEqualSign | I64GreaterEqualSign | F64GreaterEqual | F32GreaterEqual => {
                    impl_binary_inst!(self, greater_than_equal)
                }
                I32GreaterThanSign | I64GreaterThanSign | F32GreaterThan | F64GreaterThan => {
                    impl_binary_inst!(self, greater_than)
                }
                I32GreaterThanUnsign | I64GreaterThanUnSign => {
                    impl_binary_inst!(self, greater_than_unsign)
                }
                I32GreaterEqualUnsign | I64GreaterEqualUnSign => {
                    impl_binary_inst!(self, greater_than_equal_unsign)
                }
                Equal | I64Equal | F32Equal | F64Equal => impl_binary_inst!(self, equal),
                NotEqual | I64NotEqual | F32NotEqual | F64NotEqual => {
                    impl_binary_inst!(self, not_equal)
                }
                I32Or | I64Or => impl_binary_inst!(self, or),
                I32Xor | I64Xor => impl_binary_inst!(self, xor),
                I32And | I64And => impl_binary_inst!(self, and),
                I32ShiftLeft | I64ShiftLeft => impl_binary_inst!(self, shift_left),
                I32ShiftRIghtSign | I64ShiftRightSign => impl_binary_inst!(self, shift_right_sign),
                I32ShiftRightUnsign | I64ShiftRightUnsign => {
                    impl_binary_inst!(self, shift_right_unsign)
                }
                I32WrapI64 => {
                    let i = &self.stack.pop_value_ext();
                    match i {
                        Values::I64(n) => {
                            let result = (*n % 2_i64.pow(32)) as i32;
                            self.stack
                                .push(StackEntry::new_value(Values::I32(result)))?;
                        }
                        x => unreachable!("Expected i64 value, got {:?}", x),
                    }
                }
                I32RotateLeft | I64RotateLeft => impl_binary_inst!(self, wasm_rotate_left),
                I32RotateRight | I64RotateRight => impl_binary_inst!(self, wasm_rotate_right),
                I32CountLeadingZero | I64CountLeadingZero => {
                    impl_unary_inst!(self, count_leading_zero)
                }
                I32CountTrailingZero | I64CountTrailingZero => {
                    impl_unary_inst!(self, count_trailing_zero)
                }
                I32CountNonZero | I64CountNonZero => impl_unary_inst!(self, pop_count),
                I32EqualZero | I64EqualZero => impl_unary_inst!(self, equal_zero),

                I32Load(_, offset) => impl_load_inst!(32, self, offset, ValueTypes::I32),
                I64Load(_, offset) => impl_load_inst!(64, self, offset, ValueTypes::I64),
                F32Load(_, offset) => impl_load_inst!(32, self, offset, ValueTypes::F32),
                F64Load(_, offset) => impl_load_inst!(64, self, offset, ValueTypes::F64),
                I32Load8Unsign(_, offset) => impl_load_inst!(8, self, offset, ValueTypes::I32),
                I32Load8Sign(_, offset) => impl_load_inst!(8, self, offset, ValueTypes::I32),
                I32Load16Unsign(_, offset) => impl_load_inst!(16, self, offset, ValueTypes::I32),
                I32Load16Sign(_, offset) => impl_load_inst!(16, self, offset, ValueTypes::I32),
                I64Load8Unsign(_, offset) => impl_load_inst!(8, self, offset, ValueTypes::I64),
                I64Load8Sign(_, offset) => impl_load_inst!(8, self, offset, ValueTypes::I64),
                I64Load16Unsign(_, offset) => impl_load_inst!(16, self, offset, ValueTypes::I64),
                I64Load16Sign(_, offset) => impl_load_inst!(16, self, offset, ValueTypes::I64),
                I64Load32Sign(_, offset) => impl_load_inst!(32, self, offset, ValueTypes::I64),
                I64Load32Unsign(_, offset) => impl_load_inst!(32, self, offset, ValueTypes::I64),

                I32Store(_, offset) => impl_store_inst!(32, self, offset),
                F32Store(_, offset) => impl_store_inst!(32, self, offset),
                I64Store(_, offset) => impl_store_inst!(64, self, offset),
                F64Store(_, offset) => impl_store_inst!(64, self, offset),
                I32Store8(_, offset) => impl_store_inst!(8, self, offset),
                I32Store16(_, offset) => impl_store_inst!(16, self, offset),
                I64Store8(_, offset) => impl_store_inst!(8, self, offset),
                I64Store16(_, offset) => impl_store_inst!(16, self, offset),
                I64Store32(_, offset) => impl_store_inst!(32, self, offset),

                F32Copysign | F64Copysign => impl_binary_inst!(self, copy_sign),
                F32Abs | F64Abs => impl_unary_inst!(self, abs),
                F64Neg | F32Neg => impl_unary_inst!(self, neg),
                MemorySize => {
                    unimplemented!();
                }
                MemoryGrow => {
                    let page_size = self.store.size_by_pages();
                    let n = self.stack.pop_value_ext_i32() as u32;
                    let result = match self.store.memory_grow(n) {
                        Ok(()) => (page_size as i32),
                        Err(Trap::FailToGrow) => -1,
                        _ => unreachable!(),
                    };
                    self.stack
                        .push(StackEntry::new_value(Values::I32(result)))?;
                }

                I64ExtendUnsignI32 => impl_unary_inst!(self, extend_to_i64),

                F32ConvertSignI32 => impl_unary_inst!(self, convert_sign_i32_to_f32),
                F32ConvertUnsignI32 => impl_unary_inst!(self, convert_unsign_i32_to_f32),
                F64ConvertSignI64 => impl_unary_inst!(self, convert_sign_i64_to_f64),
                F64ConvertUnsignI64 => impl_unary_inst!(self, convert_unsign_i64_to_f64),
                F64ConvertSignI32 => impl_unary_inst!(self, convert_sign_i32_to_f64),
                F64ConvertUnsignI32 => impl_unary_inst!(self, convert_unsign_i32_to_f64),
                F32ConvertSignI64 => impl_unary_inst!(self, convert_sign_i64_to_f32),
                F32ConvertUnsignI64 => impl_unary_inst!(self, convert_unsign_i64_to_f32),

                I32TruncSignF32 => impl_try_unary_inst!(self, trunc_f32_to_sign_i32),
                I32TruncUnsignF32 => impl_try_unary_inst!(self, trunc_f32_to_unsign_i32),
                I64TruncSignF64 => impl_try_unary_inst!(self, trunc_f64_to_sign_i64),
                I64TruncUnsignF64 => impl_try_unary_inst!(self, trunc_f64_to_unsign_i64),
                I32TruncSignF64 => impl_try_unary_inst!(self, trunc_f64_to_sign_i32),
                I32TruncUnsignF64 => impl_try_unary_inst!(self, trunc_f64_to_unsign_i32),
                I64TruncSignF32 => impl_try_unary_inst!(self, trunc_f32_to_sign_i64),
                I64TruncUnsignF32 => impl_try_unary_inst!(self, trunc_f32_to_unsign_i64),

                F64PromoteF32 => impl_unary_inst!(self, promote_f32_to_f64),
                F32DemoteF64 => impl_unary_inst!(self, demote_f64_to_f32),

                I64ExtendSignI32 | I32ReinterpretF32 | I64ReinterpretF64 | F32ReinterpretI32
                | F64ReinterpretI64 => {
                    unimplemented!("{:?}", expression);
                }
                RuntimeValue(t) => unreachable!("{:?}", t),
            };
        }
        Ok(())
    }

    fn evaluate_frame(
        &mut self,
        instructions: &mut Instructions,
        function_type: FunctionType,
    ) -> Result<()> {
        self.evaluate_instructions(instructions)?;
        let mut returns = vec![];
        for _ in 0..function_type.get_return_count() {
            returns.push(self.stack.pop_value()?);
        }
        self.stack.update_frame_ptr();
        for v in returns.iter() {
            self.stack.push(StackEntry::new_value(v.clone()))?;
        }
        Ok(())
    }

    fn expand_frame(&mut self, function_idx: usize, arguments: Vec<Values>) -> Result<()> {
        let function_instance = self.store.call(function_idx)?;
        let own_type = match function_instance.function_type {
            Ok(ref t) => Some(t.to_owned()),
            _ => None,
        };
        let (expressions, local_types) = function_instance.call();
        let mut locals = arguments;
        for local in local_types {
            let v = match local {
                ValueTypes::I32 => Values::I32(0),
                ValueTypes::I64 => Values::I64(0),
                ValueTypes::F32 => Values::F32(0.0),
                ValueTypes::F64 => Values::F64(0.0),
                _ => unreachable!(),
            };
            locals.push(v);
        }
        let frame = StackEntry::new_fram(Frame {
            locals,
            expressions,
            return_ptr: self.stack.stack_ptr,
            function_idx,
            table_addresses: vec![0],
            types: self.store.gather_function_types(),
            own_type,
        });
        self.stack.push(frame)?;
        Ok(())
    }

    fn evaluate(&mut self) -> Result<()> {
        let mut result = None;
        while !self.stack.is_empty() {
            let popped = match self.stack.pop() {
                Ok(p) => p,
                Err(_) => {
                    break;
                }
            };
            match *popped {
                StackEntry::Value(ref v) => {
                    result = Some(StackEntry::new_value(v.to_owned()));
                    break;
                }
                StackEntry::Frame(ref frame) => {
                    self.stack.frame_ptr.push(frame.return_ptr);
                    let frame = frame.clone();
                    let own_type = frame.own_type;
                    for local in frame.locals {
                        self.stack.push(StackEntry::new_value(local))?;
                    }
                    let mut insts = Instructions::new(
                        frame.expressions,
                        frame.table_addresses.to_owned(),
                        frame.types.to_owned(),
                    );
                    self.evaluate_frame(&mut insts, own_type?)?;
                }
                StackEntry::Empty => unreachable!("Invalid popping stack."),
            }
        }
        if let Some(v) = result {
            self.stack.push(v)?;
        };
        Ok(())
    }

    pub fn run(&mut self, invoke: &str, arguments: Vec<Values>) -> String {
        let start_idx = self.store.get_function_idx(invoke);
        let _ = self.expand_frame(start_idx, arguments);

        match self.evaluate() {
            Ok(_) => match self.stack.pop_value() {
                Ok(v) => String::from(v),
                Err(_) => "".to_owned(),
            },
            Err(err) => String::from(err),
        }
    }
}
