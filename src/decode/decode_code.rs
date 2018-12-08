macro_rules! impl_decode_code {
  ($name: ident) => {
    impl $name {
      fn decode_memory_inst(&mut self) -> Result<(u32, u32)> {
        let align = self.decode_leb128_u32();
        let offset = self.decode_leb128_u32();
        match (align, offset) {
          (Ok(align), Ok(offset)) => Ok((align as u32, offset as u32)),
          (Err(Trap::BitshiftOverflow), _) | (_, Err(Trap::BitshiftOverflow)) => {
            Err(Trap::MemoryAccessOutOfBounds)
          }
          _ => Err(Trap::Unknown),
        }
      }

      fn decode_section_code_internal(&mut self) -> Result<Vec<$crate::inst::Inst>> {
        use code::{Code, ValueTypes};
        use inst::Inst;
        let mut expressions = vec![];
        while !Code::is_else_or_end(self.peek()) {
          let code = Code::from(self.next());
          match code {
            Code::Reserved => unreachable!(),
            Code::Unreachable => expressions.push(Inst::Unreachable),
            Code::Nop => expressions.push(Inst::Nop),
            Code::Block => {
              let block_type = Inst::RuntimeValue(ValueTypes::from(self.next()));
              let mut instructions = self.decode_section_code_internal()?;
              expressions.push(Inst::Block(
                (2 /* If inst + Type of block */ + instructions.len()) as u32,
              ));
              expressions.push(block_type);
              expressions.append(&mut instructions);
            }
            Code::Loop => {
              expressions.push(Inst::Loop);
              expressions.push(Inst::RuntimeValue(ValueTypes::from(self.next())));
              let mut instructions = self.decode_section_code_internal()?;
              expressions.append(&mut instructions);
            }
            Code::If => {
              let block_type = Inst::RuntimeValue(ValueTypes::from(self.next()));
              let mut if_insts = self.decode_section_code_internal()?;
              let last = if_insts.last().map(|x| x.clone());

              let mut else_insts = match last {
                Some(Inst::Else) => self.decode_section_code_internal()?,
                Some(Inst::End) => vec![],
                x => unreachable!("{:?}", x),
              };
              expressions.push(Inst::If(
                (2 /* If inst + Type of block */ + if_insts.len()) as u32,
                else_insts.len() as u32,
              ));
              expressions.push(block_type);
              expressions.append(&mut if_insts);
              expressions.append(&mut else_insts);
            }
            Code::Br => expressions.push(Inst::Br(self.decode_leb128_u32()?)),
            Code::BrIf => expressions.push(Inst::BrIf(self.decode_leb128_u32()?)),
            Code::BrTable => {
              let len = self.decode_leb128_u32()?;
              let tables = (0..len)
                .map(|_| self.decode_leb128_u32().expect("Can't decode integer."))
                .collect::<Vec<_>>();
              let idx = self.decode_leb128_u32()?;
              expressions.push(Inst::BrTable(tables, idx))
            }
            Code::Return => expressions.push(Inst::Return),
            Code::Call => expressions.push(Inst::Call(self.decode_leb128_u32()? as usize)),
            Code::CallIndirect => {
              expressions.push(Inst::CallIndirect(self.decode_leb128_u32()?));
              self.next(); // Drop code 0x00.
            }

            // NOTE: Consume at decoding "If" instructions.
            Code::End | Code::Else => unreachable!("{:?}", code),
            Code::ConstI32 => expressions.push(Inst::I32Const(self.decode_leb128_i32()?)),
            Code::ConstI64 => expressions.push(Inst::I64Const(self.decode_leb128_i64()?)),
            Code::F32Const => expressions.push(Inst::F32Const(self.decode_f32()?)),
            Code::F64Const => expressions.push(Inst::F64Const(self.decode_f64()?)),
            // NOTE: It might be need to decode as LEB128 integer, too.
            Code::GetLocal => expressions.push(Inst::GetLocal(self.decode_leb128_u32()?)),
            Code::SetLocal => expressions.push(Inst::SetLocal(self.decode_leb128_u32()?)),
            Code::TeeLocal => expressions.push(Inst::TeeLocal(self.decode_leb128_u32()?)),
            Code::GetGlobal => expressions.push(Inst::GetGlobal(self.decode_leb128_u32()?)),
            Code::SetGlobal => expressions.push(Inst::SetGlobal(self.decode_leb128_u32()?)),
            Code::DropInst => expressions.push(Inst::DropInst),

            Code::I32Load => {
              match self.decode_memory_inst() {
                Ok((align, offset)) => expressions.push(Inst::I32Load(align, offset)),
                Err(err) => return Err(err),
              };
            }
            Code::I64Load => {
              let (align, offset) = self.decode_memory_inst()?;
              expressions.push(Inst::I64Load(align, offset));
            }
            Code::F32Load => {
              let (align, offset) = self.decode_memory_inst()?;
              expressions.push(Inst::F32Load(align, offset));
            }
            Code::F64Load => {
              let (align, offset) = self.decode_memory_inst()?;
              expressions.push(Inst::F64Load(align, offset));
            }
            Code::I32Load8Sign => {
              let (align, offset) = self.decode_memory_inst()?;
              expressions.push(Inst::I32Load8Sign(align, offset));
            }
            Code::I32Load8Unsign => {
              let (align, offset) = self.decode_memory_inst()?;
              expressions.push(Inst::I32Load8Unsign(align, offset));
            }
            Code::I32Load16Sign => {
              let (align, offset) = self.decode_memory_inst()?;
              expressions.push(Inst::I32Load16Sign(align, offset));
            }
            Code::I32Load16Unsign => {
              let (align, offset) = self.decode_memory_inst()?;
              expressions.push(Inst::I32Load16Unsign(align, offset));
            }
            Code::I64Load8Sign => {
              let (align, offset) = self.decode_memory_inst()?;
              expressions.push(Inst::I64Load8Sign(align, offset));
            }
            Code::I64Load8Unsign => {
              let (align, offset) = self.decode_memory_inst()?;
              expressions.push(Inst::I64Load8Unsign(align, offset));
            }
            Code::I64Load16Sign => {
              let (align, offset) = self.decode_memory_inst()?;
              expressions.push(Inst::I64Load16Sign(align, offset));
            }
            Code::I64Load16Unsign => {
              let (align, offset) = self.decode_memory_inst()?;
              expressions.push(Inst::I64Load16Unsign(align, offset));
            }
            Code::I64Load32Sign => {
              let (align, offset) = self.decode_memory_inst()?;
              expressions.push(Inst::I64Load32Sign(align, offset));
            }
            Code::I64Load32Unsign => {
              let (align, offset) = self.decode_memory_inst()?;
              expressions.push(Inst::I64Load32Unsign(align, offset));
            }
            Code::I32Store => {
              let (align, offset) = self.decode_memory_inst()?;
              expressions.push(Inst::I32Store(align, offset));
            }
            Code::I64Store => {
              let (align, offset) = self.decode_memory_inst()?;
              expressions.push(Inst::I64Store(align, offset));
            }
            Code::F32Store => {
              let (align, offset) = self.decode_memory_inst()?;
              expressions.push(Inst::F32Store(align, offset));
            }
            Code::F64Store => {
              let (align, offset) = self.decode_memory_inst()?;
              expressions.push(Inst::F64Store(align, offset));
            }
            Code::I32Store8 => {
              let (align, offset) = self.decode_memory_inst()?;
              expressions.push(Inst::I32Store8(align, offset));
            }
            Code::I32Store16 => {
              let (align, offset) = self.decode_memory_inst()?;
              expressions.push(Inst::I32Store16(align, offset));
            }
            Code::I64Store8 => {
              let (align, offset) = self.decode_memory_inst()?;
              expressions.push(Inst::I64Store8(align, offset));
            }
            Code::I64Store16 => {
              let (align, offset) = self.decode_memory_inst()?;
              expressions.push(Inst::I64Store16(align, offset));
            }
            Code::I64Store32 => {
              let (align, offset) = self.decode_memory_inst()?;
              expressions.push(Inst::I64Store32(align, offset));
            }
            Code::MemorySize => {
              self.next()?; // Drop 0x00;
              expressions.push(Inst::MemorySize);
            }
            Code::MemoryGrow => {
              self.next()?; // Drop 0x00;
              expressions.push(Inst::MemoryGrow);
            }
            Code::I32CountLeadingZero => expressions.push(Inst::I32CountLeadingZero),
            Code::I32CountTrailingZero => expressions.push(Inst::I32CountTrailingZero),
            Code::I32CountNonZero => expressions.push(Inst::I32CountNonZero),
            Code::I32Add => expressions.push(Inst::I32Add),
            Code::I32Sub => expressions.push(Inst::I32Sub),
            Code::I32Mul => expressions.push(Inst::I32Mul),
            Code::I32DivSign => expressions.push(Inst::I32DivSign),
            Code::I32DivUnsign => expressions.push(Inst::I32DivUnsign),
            Code::I32RemSign => expressions.push(Inst::I32RemSign),
            Code::I32RemUnsign => expressions.push(Inst::I32RemUnsign),
            Code::I32And => expressions.push(Inst::I32And),
            Code::I32Or => expressions.push(Inst::I32Or),
            Code::I32Xor => expressions.push(Inst::I32Xor),
            Code::I32ShiftLeft => expressions.push(Inst::I32ShiftLeft),
            Code::I32ShiftRIghtSign => expressions.push(Inst::I32ShiftRIghtSign),
            Code::I32ShiftRightUnsign => expressions.push(Inst::I32ShiftRightUnsign),
            Code::I32RotateLeft => expressions.push(Inst::I32RotateLeft),
            Code::I32RotateRight => expressions.push(Inst::I32RotateRight),
            Code::I64CountLeadingZero => expressions.push(Inst::I64CountLeadingZero),
            Code::I64CountTrailingZero => expressions.push(Inst::I64CountTrailingZero),
            Code::I64CountNonZero => expressions.push(Inst::I64CountNonZero),
            Code::I64Add => expressions.push(Inst::I64Add),
            Code::I64Sub => expressions.push(Inst::I64Sub),
            Code::I64Mul => expressions.push(Inst::I64Mul),
            Code::I64DivSign => expressions.push(Inst::I64DivSign),
            Code::I64DivUnsign => expressions.push(Inst::I64DivUnsign),
            Code::I64RemSign => expressions.push(Inst::I64RemSign),
            Code::I64RemUnsign => expressions.push(Inst::I64RemUnsign),
            Code::I64And => expressions.push(Inst::I64And),
            Code::I64Or => expressions.push(Inst::I64Or),
            Code::I64Xor => expressions.push(Inst::I64Xor),
            Code::I64ShiftLeft => expressions.push(Inst::I64ShiftLeft),
            Code::I64ShiftRightSign => expressions.push(Inst::I64ShiftRightSign),
            Code::I64ShiftRightUnsign => expressions.push(Inst::I64ShiftRightUnsign),
            Code::I64RotateLeft => expressions.push(Inst::I64RotateLeft),
            Code::I64RotateRight => expressions.push(Inst::I64RotateRight),

            Code::I64EqualZero => expressions.push(Inst::I64EqualZero),
            Code::I64Equal => expressions.push(Inst::I64Equal),
            Code::I64NotEqual => expressions.push(Inst::I64NotEqual),
            Code::I64LessThanSign => expressions.push(Inst::I64LessThanSign),
            Code::I64LessThanUnSign => expressions.push(Inst::I64LessThanUnSign),
            Code::I64GreaterThanSign => expressions.push(Inst::I64GreaterThanSign),
            Code::I64GreaterThanUnSign => expressions.push(Inst::I64GreaterThanUnSign),
            Code::I64LessEqualSign => expressions.push(Inst::I64LessEqualSign),
            Code::I64LessEqualUnSign => expressions.push(Inst::I64LessEqualUnSign),
            Code::I64GreaterEqualSign => expressions.push(Inst::I64GreaterEqualSign),
            Code::I64GreaterEqualUnSign => expressions.push(Inst::I64GreaterEqualUnSign),

            Code::I32WrapI64 => expressions.push(Inst::I32WrapI64),
            Code::I32EqualZero => expressions.push(Inst::I32EqualZero),
            Code::Equal => expressions.push(Inst::Equal),
            Code::NotEqual => expressions.push(Inst::NotEqual),
            Code::LessThanSign => expressions.push(Inst::LessThanSign),
            Code::LessThanUnsign => expressions.push(Inst::LessThanUnsign),
            Code::GreaterThanSign => expressions.push(Inst::I32GreaterThanSign),
            Code::I32GreaterThanUnsign => expressions.push(Inst::I32GreaterThanUnsign),
            Code::I32LessEqualSign => expressions.push(Inst::I32LessEqualSign),
            Code::I32LessEqualUnsign => expressions.push(Inst::I32LessEqualUnsign),
            Code::I32GreaterEqualSign => expressions.push(Inst::I32GreaterEqualSign),
            Code::I32GreaterEqualUnsign => expressions.push(Inst::I32GreaterEqualUnsign),

            Code::F32Equal => expressions.push(Inst::F32Equal),
            Code::F32NotEqual => expressions.push(Inst::F32NotEqual),
            Code::F32LessThan => expressions.push(Inst::F32LessThan),
            Code::F32GreaterThan => expressions.push(Inst::F32GreaterThan),
            Code::F32LessEqual => expressions.push(Inst::F32LessEqual),
            Code::F32GreaterEqual => expressions.push(Inst::F32GreaterEqual),
            Code::F64Equal => expressions.push(Inst::F64Equal),
            Code::F64NotEqual => expressions.push(Inst::F64NotEqual),
            Code::F64LessThan => expressions.push(Inst::F64LessThan),
            Code::F64GreaterThan => expressions.push(Inst::F64GreaterThan),
            Code::F64LessEqual => expressions.push(Inst::F64LessEqual),
            Code::F64GreaterEqual => expressions.push(Inst::F64GreaterEqual),

            Code::F32Abs => expressions.push(Inst::F32Abs),
            Code::F32Neg => expressions.push(Inst::F32Neg),
            Code::F32Ceil => expressions.push(Inst::F32Ceil),
            Code::F32Floor => expressions.push(Inst::F32Floor),
            Code::F32Trunc => expressions.push(Inst::F32Trunc),
            Code::F32Nearest => expressions.push(Inst::F32Nearest),
            Code::F32Sqrt => expressions.push(Inst::F32Sqrt),
            Code::F32Add => expressions.push(Inst::F32Add),
            Code::F32Sub => expressions.push(Inst::F32Sub),
            Code::F32Mul => expressions.push(Inst::F32Mul),
            Code::F32Div => expressions.push(Inst::F32Div),
            Code::F32Min => expressions.push(Inst::F32Min),
            Code::F32Max => expressions.push(Inst::F32Max),
            Code::F32Copysign => expressions.push(Inst::F32Copysign),

            Code::F64Abs => expressions.push(Inst::F64Abs),
            Code::F64Neg => expressions.push(Inst::F64Neg),
            Code::F64Ceil => expressions.push(Inst::F64Ceil),
            Code::F64Floor => expressions.push(Inst::F64Floor),
            Code::F64Trunc => expressions.push(Inst::F64Trunc),
            Code::F64Nearest => expressions.push(Inst::F64Nearest),
            Code::F64Sqrt => expressions.push(Inst::F64Sqrt),
            Code::F64Add => expressions.push(Inst::F64Add),
            Code::F64Sub => expressions.push(Inst::F64Sub),
            Code::F64Mul => expressions.push(Inst::F64Mul),
            Code::F64Div => expressions.push(Inst::F64Div),
            Code::F64Min => expressions.push(Inst::F64Min),
            Code::F64Max => expressions.push(Inst::F64Max),
            Code::F64Copysign => expressions.push(Inst::F64Copysign),

            Code::I32TruncSignF32 => expressions.push(Inst::I32TruncSignF32),
            Code::I32TruncUnsignF32 => expressions.push(Inst::I32TruncUnsignF32),
            Code::I32TruncSignF64 => expressions.push(Inst::I32TruncSignF64),
            Code::I32TruncUnsignF64 => expressions.push(Inst::I32TruncUnsignF64),
            Code::I64ExtendSignI32 => expressions.push(Inst::I64ExtendSignI32),
            Code::I64ExtendUnsignI32 => expressions.push(Inst::I64ExtendUnsignI32),
            Code::I64TruncSignF32 => expressions.push(Inst::I64TruncSignF32),
            Code::I64TruncUnsignF32 => expressions.push(Inst::I64TruncUnsignF32),
            Code::I64TruncSignF64 => expressions.push(Inst::I64TruncSignF64),
            Code::I64TruncUnsignF64 => expressions.push(Inst::I64TruncUnsignF64),
            Code::F32ConvertSignI32 => expressions.push(Inst::F32ConvertSignI32),
            Code::F32ConvertUnsignI32 => expressions.push(Inst::F32ConvertUnsignI32),
            Code::F32ConvertSignI64 => expressions.push(Inst::F32ConvertSignI64),
            Code::F32ConvertUnsignI64 => expressions.push(Inst::F32ConvertUnsignI64),
            Code::F32DemoteF64 => expressions.push(Inst::F32DemoteF64),
            Code::F64ConvertSignI32 => expressions.push(Inst::F64ConvertSignI32),
            Code::F64ConvertUnsignI32 => expressions.push(Inst::F64ConvertUnsignI32),
            Code::F64ConvertSignI64 => expressions.push(Inst::F64ConvertSignI64),
            Code::F64ConvertUnsignI64 => expressions.push(Inst::F64ConvertUnsignI64),
            Code::F64PromoteF32 => expressions.push(Inst::F64PromoteF32),
            Code::I32ReinterpretF32 => expressions.push(Inst::I32ReinterpretF32),
            Code::I64ReinterpretF64 => expressions.push(Inst::I64ReinterpretF64),
            Code::F32ReinterpretI32 => expressions.push(Inst::F32ReinterpretI32),
            Code::F64ReinterpretI64 => expressions.push(Inst::F64ReinterpretI64),

            Code::Select => expressions.push(Inst::Select),
          };
        }
        match Code::from(self.next()) {
          Code::Else => expressions.push(Inst::Else),
          Code::End => expressions.push(Inst::End),
          x => unreachable!("{:?}", x),
        }
        Ok(expressions)
      }
    }
  };
}
