use decode::Byte;
use frame::Frame;
use inst::Inst;
use stack::{Label, LabelKind, Stack, StackEntry, STACK_ENTRY_KIND_FRAME, STACK_ENTRY_KIND_LABEL};
use store::Store;
use trap::{Result, Trap};
use value::Values;
use value_type::ValueTypes;

macro_rules! impl_load_inst {
    ($load_data_width: expr, $self: ident, $offset: ident, $value_kind: expr) => {{
        let width = $load_data_width / 8;
        let i = $self.stack.pop_value_ext_i32() as u32;
        let (effective_address, overflowed) = i.overflowing_add(*$offset);
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
        let (effective_address, overflowed) = i.overflowing_add(*$offset);
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

    fn get_local(&mut self, idx: u32) -> Result<()> {
        let frame_ptr = self.stack.get_frame_ptr();
        let index = (idx as usize) + frame_ptr + 1;
        let value = self.stack.get(index)?;
        self.stack.push(value)?;
        Ok(())
    }

    fn set_local(&mut self, idx: u32) -> Result<()> {
        let value = self.stack.pop().map(|s| s.to_owned())?;
        let frame_ptr = self.stack.get_frame_ptr();
        self.stack.set((idx as usize) + frame_ptr + 1, value)?;
        Ok(())
    }

    fn tee_local(&mut self, idx: u32) -> Result<()> {
        let value = self.stack.pop().map(|s| s.to_owned())?;
        self.stack.push(value.clone())?;
        let frame_ptr = self.stack.get_frame_ptr();
        self.stack.set((idx as usize) + frame_ptr + 1, value)?;
        Ok(())
    }

    fn get_global(&mut self, idx: u32) -> Result<()> {
        let value = self.store.get_global(idx)?.to_owned();
        self.stack.push(StackEntry::new_value(value))?;
        Ok(())
    }

    fn set_global(&mut self, idx: u32) -> Result<()> {
        let value = self.stack.pop_value_ext();
        self.store.set_global(idx, value);
        Ok(())
    }

    fn evaluate_instructions(
        &mut self,
        frame: &Frame, /* TODO: Consider to use RefCell type. */
    ) -> Result<()> {
        use self::Inst::*;
        while let Some(expression) = frame.pop_ref() {
            match expression {
                Unreachable => {
                    unimplemented!("{:?}", expression);
                }
                Return => {
                    let mut buf_values = self.stack.pop_until(&STACK_ENTRY_KIND_FRAME)?;
                    self.stack.push_entries(&mut buf_values)?;
                    frame.jump_to_last();
                    break;
                }
                Else | End => {
                    if frame.is_next_empty() {
                        break;
                    } else {
                        let mut buf_values = self.stack.pop_until(&STACK_ENTRY_KIND_LABEL)?;
                        let buf_label = self.stack.pop_label_ext();
                        let label = match &buf_label {
                            Label {
                                source_instruction: LabelKind::LoopContinuation,
                                ..
                            } => self.stack.pop_label_ext(),
                            _ => buf_label,
                        };
                        self.stack.push_entries(&mut buf_values)?;
                        frame.jump_to(label.continuation);
                    }
                }
                Nop => {}
                Block(size) => {
                    // Size = 10 = 1(Block) + 1(BlockType) + 7(Instructions) + 1(End)
                    // In case of ptr of instructions starts by 5,
                    //
                    // [05] Block                   | <- start_of_control
                    // [06] Block_type              | <- instructions.ptr
                    //        Instructions * 6      |
                    // [13]   Last Instruction      |
                    // [14] End                     |
                    // [15] NextInstruction         |  <- continuation
                    let start_of_label = frame.get_start_of_label();
                    let continuation = start_of_label + size;
                    let block_type = frame.pop_runtime_type()?;
                    let label = StackEntry::new_label(continuation, block_type, LabelKind::Block);
                    self.stack.push(label)?;
                }
                Loop(size) => {
                    // Size = 10 = 1(Loop) + 1(BlockType) + 7(Instructions) + 1(End)
                    // In case for ptr of frame starts by 5,
                    //
                    // [05] Loop                    | <- continuation
                    // [06] Block_type              | <- frame.ptr
                    //        Instructions * 6      |
                    // [13]   Last Instruction      |
                    // [14] End                     | <- frame.ptr when evaluation of frame completed
                    //                              |    without any label instruction.
                    let start_of_label = frame.get_start_of_label();
                    let ptr_of_end = size + start_of_label;
                    let block_type = frame.pop_runtime_type()?;
                    let label_end =
                        StackEntry::new_label(ptr_of_end, block_type.clone(), LabelKind::LoopEnd);
                    let label_continue = StackEntry::new_label(
                        start_of_label,
                        block_type,
                        LabelKind::LoopContinuation,
                    );
                    self.stack.push(label_end)?;
                    self.stack.push(label_continue)?;
                }
                If(if_size, else_size) => {
                    let cond = &self.stack.pop_value_ext();
                    let start_of_label = frame.get_start_of_label();
                    let continuation = start_of_label + if_size + else_size;
                    let block_type = frame.pop_runtime_type()?;
                    let label = StackEntry::new_label(continuation, block_type, LabelKind::If);
                    self.stack.push(label)?;
                    if cond.is_truthy() {
                    } else {
                        let start_of_else = start_of_label + if_size;
                        frame.jump_to(start_of_else);
                    }
                }
                Br(l) => {
                    let continuation = self.stack.jump_to_label(l)?;
                    frame.jump_to(continuation);
                }
                BrIf(l) => {
                    let cond = &self.stack.pop_value_ext();
                    if cond.is_truthy() {
                        let continuation = self.stack.jump_to_label(l)?;
                        frame.jump_to(continuation);
                    };
                }
                BrTable(ref tables, ref idx) => {
                    let i = self.stack.pop_value_ext_i32() as usize;
                    let l = if i < tables.len() {
                        tables.get(i)?
                    } else {
                        idx
                    };
                    let continuation = self.stack.jump_to_label(l)?;
                    frame.jump_to(continuation);
                }
                Call(idx) => {
                    let arity = self.store.get_function_instance(*idx)?.get_arity();
                    let mut arguments = vec![];
                    for _ in 0..arity {
                        arguments.push(self.stack.pop_value_ext());
                    }
                    self.stack.push_frame(&mut self.store, *idx, arguments)?;
                    break;
                }
                CallIndirect(idx) => {
                    let ta = frame.get_table_address();
                    let table = self.store.get_table_at(ta)?.clone();
                    let i = self.stack.pop_value_ext_i32();
                    if i > table.len() as i32 {
                        return Err(Trap::UndefinedElement);
                    }
                    let address = table.get_function_address(i as u32)?;
                    let mut arguments = {
                        let actual_fn_ty = self.store.get_function_type_by_instance(address)?;
                        let expect_fn_ty = self.store.get_function_type(*idx)?;
                        if &actual_fn_ty != expect_fn_ty {
                            return Err(Trap::IndirectCallTypeMismatch);
                        }
                        let mut arg = vec![];
                        for _ in 0..actual_fn_ty.get_arity() {
                            arg.push(self.stack.pop_value_ext());
                        }
                        arg
                    };

                    self.stack
                        .push_frame(&mut self.store, address as usize, arguments)?;
                    break;
                }
                GetLocal(idx) => self.get_local(*idx)?,
                SetLocal(idx) => self.set_local(*idx)?,
                TeeLocal(idx) => self.tee_local(*idx)?,
                GetGlobal(idx) => self.get_global(*idx)?,
                SetGlobal(idx) => self.set_global(*idx)?,
                I32Const(n) => self.stack.push(StackEntry::new_value(Values::I32(*n)))?,
                I64Const(n) => self.stack.push(StackEntry::new_value(Values::I64(*n)))?,
                F32Const(n) => self.stack.push(StackEntry::new_value(Values::F32(*n)))?,
                F64Const(n) => self.stack.push(StackEntry::new_value(Values::F64(*n)))?,
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

                I64ExtendUnsignI32 => impl_unary_inst!(self, extend_u32_to_i64),
                I64ExtendSignI32 => impl_unary_inst!(self, extend_i32_to_i64),
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

                I32ReinterpretF32 | I64ReinterpretF64 | F32ReinterpretI32 | F64ReinterpretI64 => {
                    impl_unary_inst!(self, reinterpret)
                }
                RuntimeValue(t) => unreachable!("Expected calculatable operation, got {:?}", t),
            };
        }
        Ok(())
    }

    fn evaluate(&mut self) -> Result<()> {
        let mut result = None;
        while !self.stack.is_empty() {
            let popped = self.stack.pop()?; //.map(|entry| *entry)?;
            match *popped {
                StackEntry::Value(ref v) => {
                    if self.stack.is_frame_ramained() {
                        let mut buf_values = self.stack.pop_until(&STACK_ENTRY_KIND_FRAME)?;
                        let frame = self.stack.pop()?;
                        self.stack.push(popped.clone())?;
                        self.stack.push_entries(&mut buf_values)?;
                        self.stack.push(frame)?;
                    } else {
                        result = Some(StackEntry::new_value(v.to_owned()));
                        break;
                    }
                }
                StackEntry::Frame(ref frame) => {
                    // NOTE: Only fresh frame should be initialization.
                    if frame.is_fresh() {
                        let prev_frame_ptr = self.stack.frame_ptr;
                        self.stack.frame_ptr = frame.return_ptr;
                        self.stack.push(StackEntry::new_pointer(prev_frame_ptr))?;
                        self.stack.push_entries(&mut frame.get_local_variables())?;
                    }
                    self.evaluate_instructions(frame)?;

                    let is_completed = frame.is_completed();
                    if !is_completed {
                        let mut next_frame = self.stack.pop_frame_ext();
                        next_frame.increment_return_ptr();
                        self.stack.push(popped.clone())?;
                        self.stack.push(StackEntry::new_frame(next_frame))?;
                        continue;
                    }
                    self.stack.decrease_pushed_frame();
                    let count_of_returns = frame.get_return_count();
                    let mut returns = vec![];
                    for _ in 0..count_of_returns {
                        returns.push(self.stack.pop_value()?);
                    }
                    self.stack.update_frame_ptr();
                    for v in returns.iter() {
                        self.stack.push(StackEntry::new_value(v.clone()))?;
                    }
                }
                StackEntry::Empty | StackEntry::Label(_) | StackEntry::Pointer(_) => {
                    unreachable!("Invalid popping stack.")
                }
            }
        }
        if let Some(v) = result {
            self.stack.push(v)?;
        };
        Ok(())
    }

    pub fn run(&mut self, invoke: &str, arguments: Vec<Values>) -> String {
        let mut arguments = arguments.to_owned();
        arguments.reverse();
        let start_idx = self.store.get_function_idx(invoke);
        let _ = self.stack.push_frame(&mut self.store, start_idx, arguments);
        match self.evaluate() {
            Ok(_) => match self.stack.pop_value() {
                Ok(v) => String::from(v),
                Err(_) => "".to_owned(),
            },
            Err(err) => String::from(err),
        }
    }
}
