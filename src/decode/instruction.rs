use super::decodable::{Peekable, SignedIntegerDecodable, U32Decodable};
use alloc::vec::Vec;
use error::runtime::Trap;
use error::{Result, WasmError};
use isa::Isa;

macro_rules! impl_decode_float {
  ($buf_ty: ty, $fn_name: ident, $bitwidth: expr) => {
    fn $fn_name(&mut self) -> $crate::error::Result<$buf_ty> {
      let mut buf = [0u8; $bitwidth];
      for i in 0..$bitwidth {
        buf[i] = self.next()?;
      }
      Ok(unsafe { core::mem::transmute::<_, $buf_ty>(buf)})
    }
  };
}

macro_rules! impl_push_raw_bytes {
  ($name: ident, $ty: ty, $width: expr) => {
    fn $name(&self, raw: $ty, expressions: &mut Vec<u8>) {
      let bytes: [u8; $width] = unsafe { core::mem::transmute(raw) };
      for byte in bytes.iter() {
        expressions.push(*byte);
      }
    }
  };
}

pub trait InstructionDecodable: U32Decodable + Peekable + SignedIntegerDecodable {
  impl_decode_float!(u32, decode_f32, 4);
  impl_decode_float!(u64, decode_f64, 8);
  impl_push_raw_bytes!(push_u32_as_bytes, u32, 4);
  impl_push_raw_bytes!(push_u64_as_bytes, u64, 8);

  fn decode_memory_parameter(&mut self) -> Result<(u32, u32)> {
    let align = self.decode_leb128_u32();
    let offset = self.decode_leb128_u32();
    match (align, offset) {
      (Ok(align), Ok(offset)) => Ok((align as u32, offset as u32)),
      (Err(WasmError::Trap(Trap::BitshiftOverflow)), _)
      | (_, Err(WasmError::Trap(Trap::BitshiftOverflow))) => {
        Err(WasmError::Trap(Trap::MemoryAccessOutOfBounds))
      }
      _ => Err(WasmError::Trap(Trap::Unknown)),
    }
  }

  fn decode_memory(&mut self, inst: u8, expressions: &mut Vec<u8>) -> Result<()> {
    let (align, offset) = self.decode_memory_parameter()?;
    expressions.push(inst);
    self.push_u32_as_bytes(align, expressions);
    self.push_u32_as_bytes(offset, expressions);
    Ok(())
  }

  fn decode_instructions(&mut self) -> Result<Vec<u8>> {
    use self::Isa::*;
    let mut expressions = vec![];
    while !Isa::is_else_or_end(self.peek()) {
      let code = self.next()?;
      match Isa::from(code) {
        // NOTE: Else and End are already consumed at decoding "If" instructions.
        Reserved | End | Else => unreachable!("{:?}", code),
        Unreachable | Nop | Return | DropInst => expressions.push(code),

        Block => {
          let block_type = self.next()?;
          let mut instructions = self.decode_instructions()?;
          let size =
            (2 /* Block inst + Type of block */ + 4 /* size of size */ + instructions.len()) as u32;
          expressions.push(code);
          self.push_u32_as_bytes(size, &mut expressions);
          expressions.push(block_type);
          expressions.append(&mut instructions);
        }
        Loop => {
          let block_type = self.next()?;
          let mut instructions = self.decode_instructions()?;
          expressions.push(code);
          expressions.push(block_type);
          expressions.append(&mut instructions);
        }
        If => {
          let block_type = self.next()?;
          let mut if_insts = self.decode_instructions()?;
          let last = *if_insts.last()?;
          let mut else_insts = match Isa::from(last) {
            Else => self.decode_instructions()?,
            End => vec![],
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

        GetLocal | SetLocal | TeeLocal | GetGlobal | SetGlobal | Br | BrIf | Call => {
          expressions.push(code);
          let idx = self.decode_leb128_u32()?;
          self.push_u32_as_bytes(idx, &mut expressions);
        }

        BrTable => {
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
        CallIndirect => {
          expressions.push(code);
          let idx = self.decode_leb128_u32()?;
          self.push_u32_as_bytes(idx, &mut expressions);
          self.next(); // Drop code 0x00.
        }

        I32Const => {
          expressions.push(code);
          let value = self.decode_leb128_i32()?;
          self.push_u32_as_bytes(value, &mut expressions);
        }
        I64Const => {
          expressions.push(code);
          let value = self.decode_leb128_i64()?;
          self.push_u64_as_bytes(value, &mut expressions);
        }
        F32Const => {
          expressions.push(code);
          let value = self.decode_f32()?;
          self.push_u32_as_bytes(value, &mut expressions);
        }
        F64Const => {
          expressions.push(code);
          let value = self.decode_f64()?;
          self.push_u64_as_bytes(value, &mut expressions);
        }

        I32Load | I64Load | F32Load | F64Load | I32Load8Sign | I32Load8Unsign | I32Load16Sign
        | I32Load16Unsign | I64Load8Sign | I64Load8Unsign | I64Load16Sign | I64Load16Unsign
        | I64Load32Sign | I64Load32Unsign | I32Store | I64Store | F32Store | F64Store
        | I32Store8 | I32Store16 | I64Store8 | I64Store16 | I64Store32 => {
          self.decode_memory(code, &mut expressions)?
        }

        MemorySize | MemoryGrow => {
          self.next()?; // Drop 0x00;
          expressions.push(code);
        }

        I32CountLeadingZero
        | I32CountTrailingZero
        | I32CountNonZero
        | I32Add
        | I32Sub
        | I32Mul
        | I32DivSign
        | I32DivUnsign
        | I32RemSign
        | I32RemUnsign
        | I32And
        | I32Or
        | I32Xor
        | I32ShiftLeft
        | I32ShiftRIghtSign
        | I32ShiftRightUnsign
        | I32RotateLeft
        | I32RotateRight
        | I64CountLeadingZero
        | I64CountTrailingZero
        | I64CountNonZero
        | I64Add
        | I64Sub
        | I64Mul
        | I64DivSign
        | I64DivUnsign
        | I64RemSign
        | I64RemUnsign
        | I64And
        | I64Or
        | I64Xor
        | I64ShiftLeft
        | I64ShiftRightSign
        | I64ShiftRightUnsign
        | I64RotateLeft
        | I64RotateRight
        | I64EqualZero
        | I64Equal
        | I64NotEqual
        | I64LessThanSign
        | I64LessThanUnSign
        | I64GreaterThanSign
        | I64GreaterThanUnSign
        | I64LessEqualSign
        | I64LessEqualUnSign
        | I64GreaterEqualSign
        | I64GreaterEqualUnSign
        | I32WrapI64
        | I32EqualZero
        | I32Equal
        | I32NotEqual
        | I32LessThanSign
        | I32LessThanUnsign
        | I32GreaterThanSign
        | I32GreaterThanUnsign
        | I32LessEqualSign
        | I32LessEqualUnsign
        | I32GreaterEqualSign
        | I32GreaterEqualUnsign
        | F32Equal
        | F32NotEqual
        | F32LessThan
        | F32GreaterThan
        | F32LessEqual
        | F32GreaterEqual
        | F64Equal
        | F64NotEqual
        | F64LessThan
        | F64GreaterThan
        | F64LessEqual
        | F64GreaterEqual
        | F32Abs
        | F32Neg
        | F32Ceil
        | F32Floor
        | F32Trunc
        | F32Nearest
        | F32Sqrt
        | F32Add
        | F32Sub
        | F32Mul
        | F32Div
        | F32Min
        | F32Max
        | F32Copysign
        | F64Abs
        | F64Neg
        | F64Ceil
        | F64Floor
        | F64Trunc
        | F64Nearest
        | F64Sqrt
        | F64Add
        | F64Sub
        | F64Mul
        | F64Div
        | F64Min
        | F64Max
        | F64Copysign
        | I32TruncSignF32
        | I32TruncUnsignF32
        | I32TruncSignF64
        | I32TruncUnsignF64
        | I64ExtendSignI32
        | I64ExtendUnsignI32
        | I64TruncSignF32
        | I64TruncUnsignF32
        | I64TruncSignF64
        | I64TruncUnsignF64
        | F32ConvertSignI32
        | F32ConvertUnsignI32
        | F32ConvertSignI64
        | F32ConvertUnsignI64
        | F32DemoteF64
        | F64ConvertSignI32
        | F64ConvertUnsignI32
        | F64ConvertSignI64
        | F64ConvertUnsignI64
        | F64PromoteF32
        | I32ReinterpretF32
        | I64ReinterpretF64
        | F32ReinterpretI32
        | F64ReinterpretI64
        | Select => expressions.push(code),
      };
    }
    let end_code = self.next()?;
    match Isa::from(end_code) {
      Else | End => expressions.push(end_code),
      x => unreachable!("{:?}", x),
    }
    Ok(expressions)
  }
}
