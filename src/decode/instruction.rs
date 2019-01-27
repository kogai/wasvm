use super::decodable::{Peekable, SignedIntegerDecodable, U32Decodable};
use alloc::vec::Vec;
use trap::{Result, Trap};

macro_rules! impl_decode_float {
  ($buf_ty: ty, $fn_name: ident, $bitwidth: expr) => {
    fn $fn_name(&mut self) -> $crate::trap::Result<$buf_ty> {
      let mut buf = [0u8; $bitwidth];
      for i in 0..$bitwidth {
        buf[i] = self.next()?;
      }
      Ok(unsafe { core::mem::transmute::<_, $buf_ty>(buf)})
    }
  };
}

pub trait InstructionDecodable: U32Decodable + Peekable + SignedIntegerDecodable {
  impl_decode_float!(u32, decode_f32, 4);
  impl_decode_float!(u64, decode_f64, 8);

  fn decode_memory_parameter(&mut self) -> Result<(u32, u32)> {
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

  fn decode_memory(&mut self, inst: u8, expressions: &mut Vec<u8>) -> Result<()> {
    let (align, offset) = self.decode_memory_parameter()?;
    expressions.push(inst);
    self.push_u32_as_bytes(align, expressions);
    self.push_u32_as_bytes(offset, expressions);
    Ok(())
  }

  // FIXME: Commonize by using macro.
  fn push_u32_as_bytes(&self, raw: u32, expressions: &mut Vec<u8>) {
    let bytes: [u8; 4] = unsafe { core::mem::transmute(raw) };
    for byte in bytes.iter() {
      expressions.push(*byte);
    }
  }

  fn push_u64_as_bytes(&self, raw: u64, expressions: &mut Vec<u8>) {
    let bytes: [u8; 8] = unsafe { core::mem::transmute(raw) };
    for byte in bytes.iter() {
      expressions.push(*byte);
    }
  }

  fn decode_instructions(&mut self) -> Result<Vec<u8>> {
    use isa::Code;
    let mut expressions = vec![];
    while !Code::is_else_or_end(self.peek()) {
      let code = self.next()?;
      match Code::from(Some(code)) {
        // NOTE: Already consumed at decoding "If" instructions.
        Code::Reserved | Code::End | Code::Else => unreachable!("{:?}", code),
        Code::Unreachable | Code::Nop | Code::Return | Code::DropInst => expressions.push(code),

        Code::Block => {
          let block_type = self.next()?;
          let mut instructions = self.decode_instructions()?;
          let size =
            (2 /* Block inst + Type of block */ + 4 /* size of size */ + instructions.len()) as u32;
          expressions.push(code);
          self.push_u32_as_bytes(size, &mut expressions);
          expressions.push(block_type);
          expressions.append(&mut instructions);
        }
        Code::Loop => {
          let block_type = self.next()?;
          let mut instructions = self.decode_instructions()?;
          expressions.push(code);
          expressions.push(block_type);
          expressions.append(&mut instructions);
        }
        Code::If => {
          let block_type = self.next()?;
          let mut if_insts = self.decode_instructions()?;
          let last = if_insts.last().cloned();

          let mut else_insts = match Code::from(last) {
            Code::Else => self.decode_instructions()?,
            Code::End => vec![],
            x => unreachable!("{:?}", x),
          };
          let size_of_if = (2 /* If inst + Type of block */ + 8 + if_insts.len()) as u32;
          let size_of_else = else_insts.len() as u32;
          expressions.push(code);
          self.push_u32_as_bytes(size_of_if, &mut expressions);
          self.push_u32_as_bytes(size_of_else, &mut expressions);
          expressions.push(block_type);
          expressions.append(&mut if_insts);
          expressions.append(&mut else_insts);
        }

        Code::GetLocal
        | Code::SetLocal
        | Code::TeeLocal
        | Code::GetGlobal
        | Code::SetGlobal
        | Code::Br
        | Code::BrIf
        | Code::Call => {
          expressions.push(code);
          let idx = self.decode_leb128_u32()?;
          self.push_u32_as_bytes(idx, &mut expressions);
        }

        Code::BrTable => {
          expressions.push(code);
          let len = self.decode_leb128_u32()?;
          self.push_u32_as_bytes(len, &mut expressions);
          for _ in 0..len {
            let idx = self.decode_leb128_u32()?;
            self.push_u32_as_bytes(idx, &mut expressions);
          }
          let idx = self.decode_leb128_u32()?;
          self.push_u32_as_bytes(idx, &mut expressions);
        }
        Code::CallIndirect => {
          expressions.push(code);
          let idx = self.decode_leb128_u32()?;
          self.push_u32_as_bytes(idx, &mut expressions);
          self.next(); // Drop code 0x00.
        }

        Code::ConstI32 => {
          expressions.push(code);
          let value = self.decode_leb128_i32()?;
          self.push_u32_as_bytes(value, &mut expressions);
        }
        Code::ConstI64 => {
          expressions.push(code);
          let value = self.decode_leb128_i64()?;
          self.push_u64_as_bytes(value, &mut expressions);
        }
        Code::F32Const => {
          expressions.push(code);
          let value = self.decode_f32()?;
          self.push_u32_as_bytes(value, &mut expressions);
        }
        Code::F64Const => {
          expressions.push(code);
          let value = self.decode_f64()?;
          self.push_u64_as_bytes(value, &mut expressions);
        }

        Code::I32Load
        | Code::I64Load
        | Code::F32Load
        | Code::F64Load
        | Code::I32Load8Sign
        | Code::I32Load8Unsign
        | Code::I32Load16Sign
        | Code::I32Load16Unsign
        | Code::I64Load8Sign
        | Code::I64Load8Unsign
        | Code::I64Load16Sign
        | Code::I64Load16Unsign
        | Code::I64Load32Sign
        | Code::I64Load32Unsign
        | Code::I32Store
        | Code::I64Store
        | Code::F32Store
        | Code::F64Store
        | Code::I32Store8
        | Code::I32Store16
        | Code::I64Store8
        | Code::I64Store16
        | Code::I64Store32 => self.decode_memory(code, &mut expressions)?,

        Code::MemorySize | Code::MemoryGrow => {
          self.next()?; // Drop 0x00;
          expressions.push(code);
        }

        Code::I32CountLeadingZero
        | Code::I32CountTrailingZero
        | Code::I32CountNonZero
        | Code::I32Add
        | Code::I32Sub
        | Code::I32Mul
        | Code::I32DivSign
        | Code::I32DivUnsign
        | Code::I32RemSign
        | Code::I32RemUnsign
        | Code::I32And
        | Code::I32Or
        | Code::I32Xor
        | Code::I32ShiftLeft
        | Code::I32ShiftRIghtSign
        | Code::I32ShiftRightUnsign
        | Code::I32RotateLeft
        | Code::I32RotateRight
        | Code::I64CountLeadingZero
        | Code::I64CountTrailingZero
        | Code::I64CountNonZero
        | Code::I64Add
        | Code::I64Sub
        | Code::I64Mul
        | Code::I64DivSign
        | Code::I64DivUnsign
        | Code::I64RemSign
        | Code::I64RemUnsign
        | Code::I64And
        | Code::I64Or
        | Code::I64Xor
        | Code::I64ShiftLeft
        | Code::I64ShiftRightSign
        | Code::I64ShiftRightUnsign
        | Code::I64RotateLeft
        | Code::I64RotateRight
        | Code::I64EqualZero
        | Code::I64Equal
        | Code::I64NotEqual
        | Code::I64LessThanSign
        | Code::I64LessThanUnSign
        | Code::I64GreaterThanSign
        | Code::I64GreaterThanUnSign
        | Code::I64LessEqualSign
        | Code::I64LessEqualUnSign
        | Code::I64GreaterEqualSign
        | Code::I64GreaterEqualUnSign
        | Code::I32WrapI64
        | Code::I32EqualZero
        | Code::Equal
        | Code::NotEqual
        | Code::LessThanSign
        | Code::LessThanUnsign
        | Code::GreaterThanSign
        | Code::I32GreaterThanUnsign
        | Code::I32LessEqualSign
        | Code::I32LessEqualUnsign
        | Code::I32GreaterEqualSign
        | Code::I32GreaterEqualUnsign
        | Code::F32Equal
        | Code::F32NotEqual
        | Code::F32LessThan
        | Code::F32GreaterThan
        | Code::F32LessEqual
        | Code::F32GreaterEqual
        | Code::F64Equal
        | Code::F64NotEqual
        | Code::F64LessThan
        | Code::F64GreaterThan
        | Code::F64LessEqual
        | Code::F64GreaterEqual
        | Code::F32Abs
        | Code::F32Neg
        | Code::F32Ceil
        | Code::F32Floor
        | Code::F32Trunc
        | Code::F32Nearest
        | Code::F32Sqrt
        | Code::F32Add
        | Code::F32Sub
        | Code::F32Mul
        | Code::F32Div
        | Code::F32Min
        | Code::F32Max
        | Code::F32Copysign
        | Code::F64Abs
        | Code::F64Neg
        | Code::F64Ceil
        | Code::F64Floor
        | Code::F64Trunc
        | Code::F64Nearest
        | Code::F64Sqrt
        | Code::F64Add
        | Code::F64Sub
        | Code::F64Mul
        | Code::F64Div
        | Code::F64Min
        | Code::F64Max
        | Code::F64Copysign
        | Code::I32TruncSignF32
        | Code::I32TruncUnsignF32
        | Code::I32TruncSignF64
        | Code::I32TruncUnsignF64
        | Code::I64ExtendSignI32
        | Code::I64ExtendUnsignI32
        | Code::I64TruncSignF32
        | Code::I64TruncUnsignF32
        | Code::I64TruncSignF64
        | Code::I64TruncUnsignF64
        | Code::F32ConvertSignI32
        | Code::F32ConvertUnsignI32
        | Code::F32ConvertSignI64
        | Code::F32ConvertUnsignI64
        | Code::F32DemoteF64
        | Code::F64ConvertSignI32
        | Code::F64ConvertUnsignI32
        | Code::F64ConvertSignI64
        | Code::F64ConvertUnsignI64
        | Code::F64PromoteF32
        | Code::I32ReinterpretF32
        | Code::I64ReinterpretF64
        | Code::F32ReinterpretI32
        | Code::F64ReinterpretI64
        | Code::Select => expressions.push(code),
      };
    }
    let end_code = self.next()?;
    match Code::from(Some(end_code)) {
      Code::Else | Code::End => expressions.push(end_code),
      x => unreachable!("{:?}", x),
    }
    Ok(expressions)
  }
}
