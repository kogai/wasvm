#[cfg(not(test))]
use alloc::prelude::*;
use alloc::vec::Vec;
use error::runtime::Trap;
use error::{Result, WasmError};
use frame::Frame;
use function::FunctionInstance;
use indice::Indice;
use isa::Isa;
use label::{Label, LabelKind};
use memory::MemoryInstances;
use module::{
    ExportDescriptor, ExternalInterface, ExternalModule, ExternalModules, InternalModule,
    ModuleDescriptor, ModuleName,
};
use stack::{Stack, StackEntry};
use store::Store;
use value::Values;
use value_type::TYPE_UNIT;

macro_rules! impl_load_inst {
    ($fn_name: ident, $load_fn: ident, $ty: ty) => {
        fn $fn_name(&self, offset: u32, load_data_width: u32, source_of_frame: &ModuleName) -> Result<$ty> {
            let memory_instances = self.get_memory_instances(source_of_frame)?;
            let width = load_data_width / 8;
            let i = self.stack.pop_value_ext_i32() as u32;
            let (effective_address, overflowed) = i.overflowing_add(offset);
            if overflowed {
                return Err(WasmError::Trap(Trap::MemoryAccessOutOfBounds));
            };
            let (ptr, overflowed) = effective_address.overflowing_add(width);
            if overflowed || memory_instances.data_size_small_than(ptr) {
                return Err(WasmError::Trap(Trap::MemoryAccessOutOfBounds));
            };
            let data = memory_instances
                .$load_fn(effective_address, ptr);
            Ok(data)
        }
    };
}

macro_rules! impl_load_to {
    ($fn_name: ident, $load_fn: ident, $path: path, $ty: ty) => {
        fn $fn_name(&mut self, offset: u32, width: u32, sign: bool, source_of_frame: &ModuleName) -> Result<()> {
            let mut value = self.$load_fn(offset, width, source_of_frame)?;
            if sign {
                let is_msb_one = value & (1 << (width - 1)) != 0;
                if is_msb_one {
                    value |= !1 << (width - 1);
                };
            }
            self.stack
                .push(StackEntry::new_value($path(value as $ty)))?;
            Ok(())
        }
    };
}

macro_rules! impl_unary_inst {
    ($op: ident) => {
        fn $op(&self) -> Result<()> {
            let popped = self.stack.pop_value_ext();
            let value = popped.$op();
            self.stack.push(StackEntry::new_value(value))?;
            Ok(())
        }
    };
}

macro_rules! impl_try_unary_inst {
    ($op: ident) => {
        fn $op(&self) -> Result<()> {
            let popped = self.stack.pop_value_ext();
            let value = popped.$op();
            match value {
                Ok(result) => {
                    self.stack.push(StackEntry::new_value(result))?;
                    Ok(())
                }
                Err(trap) => {
                    Err(trap)
                }
            }
        }
    };
}

macro_rules! impl_binary_inst {
    ($op: ident) => {
        fn $op(&self) -> Result<()> {
            let right = self.stack.pop_value_ext();
            let left = self.stack.pop_value_ext();
            let value = left.$op(&right);
            self.stack.push(StackEntry::new_value(value))?;
            Ok(())
        }
    };
}

macro_rules! impl_try_binary_inst {
    ($op: ident) => {
        fn $op(&self) -> Result<()> {
            let right = self.stack.pop_value_ext();
            let left = self.stack.pop_value_ext();
            let value = left.$op(&right);
            match value {
                Ok(result) => {
                    self.stack.push(StackEntry::new_value(result))?;
                    Ok(())
                }
                Err(trap) => {
                    Err(trap)
                }
            }
        }
    };
}

#[derive(Debug)]
pub struct ModuleInstance {
    store: Store,
    pub(crate) stack: Stack,
    internal_module: InternalModule,
    external_modules: ExternalModules,
}

impl ModuleInstance {
    impl_load_inst!(load_data_32, load_data_32, u32);
    impl_load_inst!(load_data_64, load_data_64, u64);
    impl_load_inst!(load_data_f32, load_data_f32, f32);
    impl_load_inst!(load_data_f64, load_data_f64, f64);

    impl_load_to!(load_data_to_i32, load_data_32, Values::I32, i32);
    impl_load_to!(load_data_to_i64, load_data_64, Values::I64, i64);

    impl_unary_inst!(sqrt);
    impl_unary_inst!(ceil);
    impl_unary_inst!(floor);
    impl_unary_inst!(trunc);
    impl_unary_inst!(nearest);
    impl_unary_inst!(count_leading_zero);
    impl_unary_inst!(count_trailing_zero);
    impl_unary_inst!(pop_count);
    impl_unary_inst!(equal_zero);
    impl_unary_inst!(reinterpret);
    impl_unary_inst!(abs);
    impl_unary_inst!(neg);
    impl_unary_inst!(extend_u32_to_i64);
    impl_unary_inst!(extend_i32_to_i64);
    impl_unary_inst!(convert_sign_i32_to_f32);
    impl_unary_inst!(convert_unsign_i32_to_f32);
    impl_unary_inst!(convert_sign_i64_to_f64);
    impl_unary_inst!(convert_unsign_i64_to_f64);
    impl_unary_inst!(convert_sign_i32_to_f64);
    impl_unary_inst!(convert_unsign_i32_to_f64);
    impl_unary_inst!(convert_sign_i64_to_f32);
    impl_unary_inst!(convert_unsign_i64_to_f32);
    impl_unary_inst!(promote_f32_to_f64);
    impl_unary_inst!(demote_f64_to_f32);

    impl_try_unary_inst!(trunc_f32_to_sign_i32);
    impl_try_unary_inst!(trunc_f32_to_unsign_i32);
    impl_try_unary_inst!(trunc_f64_to_sign_i64);
    impl_try_unary_inst!(trunc_f64_to_unsign_i64);
    impl_try_unary_inst!(trunc_f64_to_sign_i32);
    impl_try_unary_inst!(trunc_f64_to_unsign_i32);
    impl_try_unary_inst!(trunc_f32_to_sign_i64);
    impl_try_unary_inst!(trunc_f32_to_unsign_i64);

    impl_binary_inst!(add);
    impl_binary_inst!(sub);
    impl_binary_inst!(mul);
    impl_binary_inst!(div_f);
    impl_binary_inst!(min);
    impl_binary_inst!(max);
    impl_binary_inst!(less_than);
    impl_binary_inst!(less_than_unsign);
    impl_binary_inst!(less_than_equal);
    impl_binary_inst!(less_than_equal_unsign);
    impl_binary_inst!(greater_than);
    impl_binary_inst!(greater_than_equal);
    impl_binary_inst!(greater_than_unsign);
    impl_binary_inst!(greater_than_equal_unsign);
    impl_binary_inst!(equal);
    impl_binary_inst!(not_equal);
    impl_binary_inst!(or);
    impl_binary_inst!(xor);
    impl_binary_inst!(and);
    impl_binary_inst!(shift_left);
    impl_binary_inst!(shift_right_sign);
    impl_binary_inst!(shift_right_unsign);
    impl_binary_inst!(wasm_rotate_left);
    impl_binary_inst!(wasm_rotate_right);
    impl_binary_inst!(copy_sign);

    impl_try_binary_inst!(div_u);
    impl_try_binary_inst!(div_s);
    impl_try_binary_inst!(rem_s);
    impl_try_binary_inst!(rem_u);

    fn store(&self, data_width: u32, offset: u32, source_of_frame: &ModuleName) -> Result<()> {
        let memory_instances = self.get_memory_instances(source_of_frame)?;
        let c = self.stack.pop_value_ext();
        let width = data_width / 8;
        let i = self.stack.pop_value_ext_i32() as u32;
        let (effective_address, overflowed) = i.overflowing_add(offset);
        if overflowed {
            return Err(WasmError::Trap(Trap::MemoryAccessOutOfBounds));
        };
        let (ptr, overflowed) = effective_address.overflowing_add(width);
        if overflowed || memory_instances.data_size_small_than(ptr) {
            return Err(WasmError::Trap(Trap::MemoryAccessOutOfBounds));
        };
        memory_instances.store_data(effective_address, ptr, &c);
        Ok(())
    }

    pub fn start_index(&self) -> &Option<Indice> {
        &self.internal_module.start
    }

    pub(crate) fn new_from(
        store: Store,
        internal_module: InternalModule,
        external_modules: ExternalModules,
        stack_height: usize,
    ) -> Result<Self> {
        Ok(ModuleInstance {
            store,
            internal_module,
            stack: Stack::new(stack_height),
            external_modules,
        })
    }

    pub fn get_function_instance(&self, idx: &Indice) -> Option<FunctionInstance> {
        self.store.get_function_instance(idx)
    }

    pub fn export_module(&self) -> ExternalModule {
        ExternalModule::from(&self.store)
    }

    fn get_local(&self, idx: &Indice) -> Result<()> {
        let frame_ptr = self.stack.frame_ptr();
        let index = idx.to_usize() + frame_ptr;
        let value = self.stack.get(index)?;
        self.stack.push(value)?;
        Ok(())
    }

    fn set_local(&self, idx: &Indice) -> Result<()> {
        let value = self.stack.pop().map(|s| s.to_owned())?;
        let frame_ptr = self.stack.frame_ptr();
        self.stack.set(idx.to_usize() + frame_ptr, value)?;
        Ok(())
    }

    fn tee_local(&self, idx: &Indice) -> Result<()> {
        let value = self.stack.pop().map(|s| s.to_owned())?;
        self.stack.push(value.clone())?;
        let frame_ptr = self.stack.frame_ptr();
        self.stack.set(idx.to_usize() + frame_ptr, value)?;
        Ok(())
    }

    fn get_global(&self, idx: &Indice) -> Result<()> {
        let value = self.store.get_global(idx)?;
        self.stack.push(StackEntry::new_value(value))?;
        Ok(())
    }

    fn set_global(&mut self, idx: &Indice) -> Result<()> {
        let value = self.stack.pop_value_ext();
        self.store.set_global(idx, value);
        Ok(())
    }

    fn get_memory_instances(&self, source_of_frame: &ModuleName) -> Result<MemoryInstances> {
        Ok(match source_of_frame {
            Some(module_name) => {
                self.external_modules
                    .get(&Some(module_name.to_string()))?
                    .memory_instances
            }
            None => self.store.memory_instances.clone(),
        })
    }

    fn evaluate_instructions(&mut self, frame: &Frame) -> Result<()> {
        use self::Isa::*;
        if let FunctionInstance::HostFn(ref f) = &frame.function_instance {
            let arity = frame.function_instance.get_arity();
            let mut arguments = vec![];
            for i in 0..arity {
                self.get_local(&Indice::from(i))?;
                arguments.push(self.stack.pop_value_ext());
            }
            let results = f.call(arguments.as_slice());
            for r in results.into_iter() {
                self.stack.push(StackEntry::new_value(r))?;
            }
            return Ok(());
        }
        let source_of_frame = frame.function_instance.get_source_module_name();
        while let Some(expression) = frame.pop_ref() {
            match Isa::from(*expression) {
                Reserved => unreachable!(),
                Unreachable => return Err(WasmError::Trap(Trap::Unreachable)),
                Return => {
                    frame.jump_to_last();
                    break;
                }
                Else | End => {
                    if frame.is_next_empty() {
                        break;
                    } else {
                        let mut buf_values = self.stack.pop_until_label()?;
                        let label = self.stack.pop_label_ext();
                        if let Label {
                            source_instruction: LabelKind::If,
                            continuation,
                            ..
                        } = &label
                        {
                            frame.jump_to(*continuation);
                        };
                        self.stack.push_entries(&mut buf_values)?;
                    }
                }
                Nop => {}
                Block => {
                    // Size = 14 = 1(Block) + 1(BlockType) + 4(size) + 7(Instructions) + 1(End)
                    // In case of ptr of instructions starts by 5,
                    //
                    // [05] Block                   | <- start_of_control
                    // [06-09] Size                 |
                    // [10] Block_type              | <- instructions.ptr
                    // [11-16] Instructions * 6     |
                    // [17] Last Instruction        |
                    // [18] End                     |
                    // [19] NextInstruction         |  <- continuation
                    let start_of_label = frame.get_start_of_label();
                    let size = frame.pop_raw_u32()?;
                    let block_type = frame.pop_runtime_type()?;
                    let continuation = start_of_label + size;
                    let label = StackEntry::new_label(continuation, block_type, LabelKind::Block);
                    self.stack.push(label)?;
                }
                Loop => {
                    let start_of_label = frame.get_start_of_label();
                    let block_type = frame.pop_runtime_type()?;
                    let label_continue =
                        StackEntry::new_label(start_of_label, block_type, LabelKind::Loop);
                    self.stack.push(label_continue)?;
                }
                If => {
                    let cond = &self.stack.pop_value_ext();
                    let start_of_label = frame.get_start_of_label();
                    let if_size = frame.pop_raw_u32()?;
                    let else_size = frame.pop_raw_u32()?;
                    let continuation = start_of_label + if_size + else_size;
                    let block_type = frame.pop_runtime_type()?;
                    if cond.is_truthy() {
                        let label = StackEntry::new_label(continuation, block_type, LabelKind::If);
                        self.stack.push(label)?;
                    } else {
                        let label =
                            StackEntry::new_label(continuation, block_type, LabelKind::Else);
                        self.stack.push(label)?;
                        let start_of_else = start_of_label + if_size;
                        if else_size > 0 {
                            frame.jump_to(start_of_else);
                        } else {
                            frame.jump_to(start_of_else - 1);
                        }
                    }
                }
                Br => {
                    let label = Indice::from(frame.pop_raw_u32()?);
                    let continuation = self.stack.jump_to_label(&label)?;
                    frame.jump_to(continuation);
                }
                BrIf => {
                    let label = Indice::from(frame.pop_raw_u32()?);
                    let cond = &self.stack.pop_value_ext();
                    if cond.is_truthy() {
                        let continuation = self.stack.jump_to_label(&label)?;
                        frame.jump_to(continuation);
                    };
                }
                BrTable => {
                    let len = frame.pop_raw_u32()?;
                    let mut indices = vec![];
                    for _ in 0..len {
                        let idx = frame.pop_raw_u32()?;
                        indices.push(Indice::from(idx));
                    }
                    let idx = &Indice::from(frame.pop_raw_u32()?);
                    let i = self.stack.pop_value_ext_i32() as u32;
                    let l = if i < len {
                        indices.get(i as usize)?
                    } else {
                        idx
                    };
                    let continuation = self.stack.jump_to_label(l)?;
                    frame.jump_to(continuation);
                }
                Call => {
                    let idx = Indice::from(frame.pop_raw_u32()?);
                    let function_instance = match &source_of_frame {
                        Some(module_name) => self
                            .external_modules
                            // FIXME: Drop owning of name to search something.
                            .get_function_instance(&Some(module_name.to_owned()), idx.to_usize())
                            .map(|x| x.clone())?,
                        None => self.store.get_function_instance(&idx)?,
                    };
                    let arity = function_instance.get_arity();
                    let mut arguments = vec![];
                    for _ in 0..arity {
                        arguments.push(self.stack.pop()?);
                    }
                    let frame = Frame::new(
                        self.stack.stack_ptr(),
                        self.stack.frame_ptr(),
                        function_instance,
                        &mut arguments,
                    );
                    self.stack.push_frame(frame)?;
                    break;
                }
                CallIndirect => {
                    let idx = Indice::from(frame.pop_raw_u32()?);
                    // NOTE: Due to only single table instance allowed, `ta` always equal to 0.
                    let ta = frame.get_table_address();
                    let table = match &source_of_frame {
                        Some(module_name) => self
                            .external_modules
                            .get_table_instance(&Some(module_name.to_owned()), &ta)?,
                        None => self.store.get_table_at(&ta)?,
                    };
                    let i = self.stack.pop_value_ext_i32();
                    if i > table.len() as i32 {
                        return Err(WasmError::Trap(Trap::UndefinedElement));
                    }
                    let function_instance = table.get_function_instance(i as u32)?;
                    let mut arguments = {
                        let actual_fn_ty = function_instance.function_type_ref();
                        let expect_fn_ty = &match &source_of_frame {
                            Some(module_name) => self
                                .external_modules
                                .get_function_type(&Some(module_name.to_owned()), idx.to_u32())?,
                            None => self.store.get_function_type(&idx)?.clone(),
                        };
                        if actual_fn_ty != expect_fn_ty {
                            return Err(WasmError::Trap(Trap::IndirectCallTypeMismatch));
                        }
                        let mut arg = vec![];
                        for _ in 0..actual_fn_ty.get_arity() {
                            arg.push(self.stack.pop()?);
                        }
                        arg
                    };
                    let frame = Frame::new(
                        self.stack.stack_ptr(),
                        self.stack.frame_ptr(),
                        function_instance,
                        &mut arguments,
                    );
                    self.stack.push_frame(frame)?;
                    break;
                }
                GetLocal => {
                    let idx = Indice::from(frame.pop_raw_u32()?);
                    self.get_local(&idx)?;
                }
                SetLocal => {
                    let idx = Indice::from(frame.pop_raw_u32()?);
                    self.set_local(&idx)?;
                }
                TeeLocal => {
                    let idx = Indice::from(frame.pop_raw_u32()?);
                    self.tee_local(&idx)?
                }
                GetGlobal => {
                    let idx = Indice::from(frame.pop_raw_u32()?);
                    self.get_global(&idx)?;
                }
                SetGlobal => {
                    let idx = Indice::from(frame.pop_raw_u32()?);
                    self.set_global(&idx)?;
                }
                I32Const => {
                    let n = frame.pop_raw_u32()? as i32;
                    self.stack.push(StackEntry::new_value(Values::I32(n)))?;
                }
                I64Const => {
                    let n = frame.pop_raw_u64()? as i64;
                    self.stack.push(StackEntry::new_value(Values::I64(n)))?;
                }
                F32Const => {
                    let n = f32::from_bits(frame.pop_raw_u32()?);
                    self.stack.push(StackEntry::new_value(Values::F32(n)))?;
                }
                F64Const => {
                    let n = f64::from_bits(frame.pop_raw_u64()?);
                    self.stack.push(StackEntry::new_value(Values::F64(n)))?;
                }

                I32DivUnsign | I64DivUnsign => self.div_u()?,
                I32DivSign | I64DivSign => self.div_s()?,
                I32RemSign | I64RemSign => self.rem_s()?,
                I32RemUnsign | I64RemUnsign => self.rem_u()?,

                I32Add | I64Add | F32Add | F64Add => self.add()?,
                I32Sub | I64Sub | F32Sub | F64Sub => self.sub()?,
                I32Mul | I64Mul | F32Mul | F64Mul => self.mul()?,
                F32Div | F64Div => self.div_f()?,
                F32Min | F64Min => self.min()?,
                F32Max | F64Max => self.max()?,

                I32LessThanSign | I64LessThanSign | F32LessThan | F64LessThan => {
                    self.less_than()?
                }
                I32LessThanUnsign | I64LessThanUnSign => self.less_than_unsign()?,
                I32LessEqualSign | I64LessEqualSign | F32LessEqual | F64LessEqual => {
                    self.less_than_equal()?
                }
                I32LessEqualUnsign | I64LessEqualUnSign => self.less_than_equal_unsign()?,
                I32GreaterEqualSign | I64GreaterEqualSign | F64GreaterEqual | F32GreaterEqual => {
                    self.greater_than_equal()?
                }
                I32GreaterThanSign | I64GreaterThanSign | F32GreaterThan | F64GreaterThan => {
                    self.greater_than()?
                }
                I32GreaterThanUnsign | I64GreaterThanUnSign => self.greater_than_unsign()?,
                I32GreaterEqualUnsign | I64GreaterEqualUnSign => {
                    self.greater_than_equal_unsign()?
                }
                I32Equal | I64Equal | F32Equal | F64Equal => self.equal()?,
                I32NotEqual | I64NotEqual | F32NotEqual | F64NotEqual => self.not_equal()?,
                I32Or | I64Or => self.or()?,
                I32Xor | I64Xor => self.xor()?,
                I32And | I64And => self.and()?,
                I32ShiftLeft | I64ShiftLeft => self.shift_left()?,
                I32ShiftRIghtSign | I64ShiftRightSign => self.shift_right_sign()?,
                I32ShiftRightUnsign | I64ShiftRightUnsign => self.shift_right_unsign()?,
                I32RotateLeft | I64RotateLeft => self.wasm_rotate_left()?,
                I32RotateRight | I64RotateRight => self.wasm_rotate_right()?,
                F32Copysign | F64Copysign => self.copy_sign()?,

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
                F32Sqrt | F64Sqrt => self.sqrt()?,
                F32Ceil | F64Ceil => self.ceil()?,
                F32Floor | F64Floor => self.floor()?,
                F32Trunc | F64Trunc => self.trunc()?,
                F32Nearest | F64Nearest => self.nearest()?,

                I32CountLeadingZero | I64CountLeadingZero => self.count_leading_zero()?,
                I32CountTrailingZero | I64CountTrailingZero => self.count_trailing_zero()?,
                I32CountNonZero | I64CountNonZero => self.pop_count()?,
                I32EqualZero | I64EqualZero => self.equal_zero()?,
                F32Abs | F64Abs => self.abs()?,
                F64Neg | F32Neg => self.neg()?,
                I32ReinterpretF32 | I64ReinterpretF64 | F32ReinterpretI32 | F64ReinterpretI64 => {
                    self.reinterpret()?
                }

                I32Load => {
                    let _align = frame.pop_raw_u32()?;
                    let offset = frame.pop_raw_u32()?;
                    self.load_data_to_i32(offset, 32, true, &source_of_frame)?;
                }
                I32Load8Unsign => {
                    let _align = frame.pop_raw_u32()?;
                    let offset = frame.pop_raw_u32()?;
                    self.load_data_to_i32(offset, 8, false, &source_of_frame)?
                }
                I32Load8Sign => {
                    let _align = frame.pop_raw_u32()?;
                    let offset = frame.pop_raw_u32()?;
                    self.load_data_to_i32(offset, 8, true, &source_of_frame)?
                }
                I32Load16Unsign => {
                    let _align = frame.pop_raw_u32()?;
                    let offset = frame.pop_raw_u32()?;
                    self.load_data_to_i32(offset, 16, false, &source_of_frame)?
                }
                I32Load16Sign => {
                    let _align = frame.pop_raw_u32()?;
                    let offset = frame.pop_raw_u32()?;
                    self.load_data_to_i32(offset, 16, true, &source_of_frame)?
                }

                I64Load => {
                    let _align = frame.pop_raw_u32()?;
                    let offset = frame.pop_raw_u32()?;
                    self.load_data_to_i64(offset, 64, true, &source_of_frame)?;
                }
                I64Load8Unsign => {
                    let _align = frame.pop_raw_u32()?;
                    let offset = frame.pop_raw_u32()?;
                    self.load_data_to_i64(offset, 8, false, &source_of_frame)?
                }
                I64Load8Sign => {
                    let _align = frame.pop_raw_u32()?;
                    let offset = frame.pop_raw_u32()?;
                    self.load_data_to_i64(offset, 8, true, &source_of_frame)?
                }
                I64Load16Unsign => {
                    let _align = frame.pop_raw_u32()?;
                    let offset = frame.pop_raw_u32()?;
                    self.load_data_to_i64(offset, 16, false, &source_of_frame)?
                }
                I64Load16Sign => {
                    let _align = frame.pop_raw_u32()?;
                    let offset = frame.pop_raw_u32()?;
                    self.load_data_to_i64(offset, 16, true, &source_of_frame)?
                }
                I64Load32Unsign => {
                    let _align = frame.pop_raw_u32()?;
                    let offset = frame.pop_raw_u32()?;
                    self.load_data_to_i64(offset, 32, false, &source_of_frame)?
                }
                I64Load32Sign => {
                    let _align = frame.pop_raw_u32()?;
                    let offset = frame.pop_raw_u32()?;
                    self.load_data_to_i64(offset, 32, true, &source_of_frame)?
                }

                F32Load => {
                    let _align = frame.pop_raw_u32()?;
                    let offset = frame.pop_raw_u32()?;
                    let value = self.load_data_f32(offset, 32, &source_of_frame)?;
                    self.stack
                        .push(StackEntry::new_value(Values::F32(value as f32)))?;
                }

                F64Load => {
                    let _align = frame.pop_raw_u32()?;
                    let offset = frame.pop_raw_u32()?;
                    let value = self.load_data_f64(offset, 64, &source_of_frame)?;
                    self.stack
                        .push(StackEntry::new_value(Values::F64(value as f64)))?;
                }

                I32Store => {
                    let _align = frame.pop_raw_u32()?;
                    let offset = frame.pop_raw_u32()?;
                    self.store(32, offset, &source_of_frame)?;
                }
                F32Store => {
                    let _align = frame.pop_raw_u32()?;
                    let offset = frame.pop_raw_u32()?;
                    self.store(32, offset, &source_of_frame)?;
                }
                I64Store => {
                    let _align = frame.pop_raw_u32()?;
                    let offset = frame.pop_raw_u32()?;
                    self.store(64, offset, &source_of_frame)?;
                }
                F64Store => {
                    let _align = frame.pop_raw_u32()?;
                    let offset = frame.pop_raw_u32()?;
                    self.store(64, offset, &source_of_frame)?;
                }
                I32Store8 => {
                    let _align = frame.pop_raw_u32()?;
                    let offset = frame.pop_raw_u32()?;
                    self.store(8, offset, &source_of_frame)?;
                }
                I32Store16 => {
                    let _align = frame.pop_raw_u32()?;
                    let offset = frame.pop_raw_u32()?;
                    self.store(16, offset, &source_of_frame)?;
                }
                I64Store8 => {
                    let _align = frame.pop_raw_u32()?;
                    let offset = frame.pop_raw_u32()?;
                    self.store(8, offset, &source_of_frame)?;
                }
                I64Store16 => {
                    let _align = frame.pop_raw_u32()?;
                    let offset = frame.pop_raw_u32()?;
                    self.store(16, offset, &source_of_frame)?;
                }
                I64Store32 => {
                    let _align = frame.pop_raw_u32()?;
                    let offset = frame.pop_raw_u32()?;
                    self.store(32, offset, &source_of_frame)?;
                }

                MemorySize => {
                    let memory_instances = self.get_memory_instances(&source_of_frame)?;
                    let page_size = memory_instances.size_by_pages();
                    self.stack
                        .push(StackEntry::new_value(Values::I32(page_size as i32)))?;
                }
                MemoryGrow => {
                    let memory_instances = self.get_memory_instances(&source_of_frame)?;
                    let page_size = memory_instances.size_by_pages();
                    let n = self.stack.pop_value_ext_i32() as u32;
                    let result = match memory_instances.memory_grow(n) {
                        Ok(()) => (page_size as i32),
                        Err(WasmError::Trap(Trap::FailToGrow)) => -1,
                        _ => unreachable!(),
                    };
                    self.stack
                        .push(StackEntry::new_value(Values::I32(result)))?;
                }

                I64ExtendUnsignI32 => self.extend_u32_to_i64()?,
                I64ExtendSignI32 => self.extend_i32_to_i64()?,
                F32ConvertSignI32 => self.convert_sign_i32_to_f32()?,
                F32ConvertUnsignI32 => self.convert_unsign_i32_to_f32()?,
                F64ConvertSignI64 => self.convert_sign_i64_to_f64()?,
                F64ConvertUnsignI64 => self.convert_unsign_i64_to_f64()?,
                F64ConvertSignI32 => self.convert_sign_i32_to_f64()?,
                F64ConvertUnsignI32 => self.convert_unsign_i32_to_f64()?,
                F32ConvertSignI64 => self.convert_sign_i64_to_f32()?,
                F32ConvertUnsignI64 => self.convert_unsign_i64_to_f32()?,
                F64PromoteF32 => self.promote_f32_to_f64()?,
                F32DemoteF64 => self.demote_f64_to_f32()?,

                I32TruncSignF32 => self.trunc_f32_to_sign_i32()?,
                I32TruncUnsignF32 => self.trunc_f32_to_unsign_i32()?,
                I64TruncSignF64 => self.trunc_f64_to_sign_i64()?,
                I64TruncUnsignF64 => self.trunc_f64_to_unsign_i64()?,
                I32TruncSignF64 => self.trunc_f64_to_sign_i32()?,
                I32TruncUnsignF64 => self.trunc_f64_to_unsign_i32()?,
                I64TruncSignF32 => self.trunc_f32_to_sign_i64()?,
                I64TruncUnsignF32 => self.trunc_f32_to_unsign_i64()?,
            };
        }
        Ok(())
    }

    pub(crate) fn evaluate(&mut self) -> Result<()> {
        while !self.stack.call_stack_is_empty() {
            let frame = self.stack.pop_frame()?;
            // NOTE: Only fresh frame should be initialization.
            if frame.is_fresh() {
                let return_type = frame
                    .get_return_type()
                    .first()
                    .map_or(TYPE_UNIT, |x| x.to_owned());
                let label = StackEntry::new_label(frame.last_ptr, return_type, LabelKind::Frame);
                self.stack.frame_ptr.set(frame.return_ptr);
                self.stack.push_entries(&mut frame.get_local_variables())?;
                self.stack.push(label)?;
            }
            self.evaluate_instructions(&frame)?;

            let is_completed = frame.is_completed();
            if !is_completed {
                self.stack.push_back_frame(frame);
                continue;
            }
            let count_of_returns = frame.get_return_count();
            let mut returns = vec![];
            for _ in 0..count_of_returns {
                returns.push(StackEntry::new_value(self.stack.pop_value()?));
            }
            self.stack.update_frame_ptr(&frame);
            self.stack.push_entries(&mut returns)?;
        }
        Ok(())
    }

    fn run_internal(&mut self, invoke: &str, mut arguments: Vec<Values>) -> Result<Values> {
        match self
            .internal_module
            .get_export_by_key(invoke)
            // FIXME: Remove to owning.
            .map(|x| x.to_owned())
        {
            Some(ExternalInterface {
                descriptor: ModuleDescriptor::ExportDescriptor(ExportDescriptor::Function(idx)),
                ..
            }) => {
                let mut argument_entries = vec![];
                while let Some(argument) = arguments.pop() {
                    argument_entries.push(StackEntry::new_value(argument));
                }
                let function_instance = self.store.get_function_instance(&idx).unwrap();
                let frame = Frame::new(
                    self.stack.stack_ptr(),
                    self.stack.frame_ptr(),
                    function_instance,
                    &mut argument_entries,
                );
                let _ = self.stack.push_frame(frame);
                match self.evaluate() {
                    Ok(_) => match self.stack.pop_value() {
                        Ok(v) => Ok(v),
                        Err(_) => Ok(Values::I32(0)),
                    },
                    // Err(WasmError::Trap(Trap::StackUnderflow)) => Ok(Values::I32(0)),
                    Err(err) => Err(err),
                }
            }
            Some(ExternalInterface {
                descriptor: ModuleDescriptor::ExportDescriptor(ExportDescriptor::Global(idx)),
                ..
            }) => self.store.get_global(&idx),
            None => Err(WasmError::Trap(Trap::Notfound)),
            x => unimplemented!("{:?}", x),
        }
    }

    #[cfg(not(debug_assertions))]
    pub fn run(&mut self, invoke: &str, arguments: Vec<Values>) -> Result<Values> {
        self.run_internal(invoke, arguments)
    }

    #[cfg(debug_assertions)]
    pub fn run(&mut self, invoke: &str, arguments: Vec<Values>) -> Result<Values> {
        self.stack = Stack::new(self.stack.stack_size);
        self.run_internal(invoke, arguments)
    }
}
