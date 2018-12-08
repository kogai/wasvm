use code::{Code, ExportDescriptionCode, SectionCode, ValueTypes};
use element::Element;
use function::{FunctionInstance, FunctionType};
use global::{GlobalInstance, GlobalType};
use inst::{Inst, Instructions};
use memory::{Data, Limit, MemoryInstance};
use std::convert::From;
use std::{f32, f64};
use store::Store;
use table::{ElementType, TableInstance, TableType};
use trap::{Result, Trap};

macro_rules! leb128 {
  ($t:ty, $buf_size: ty, $fn_name: ident) => {
    fn $fn_name(&mut self) -> Result<$t> {
      let mut buf: $t = 0;
      let mut shift = 0;

      // Check whether leftmost bit is 1 or 0
      // n     = 0b11111111 = 0b01111111
      // _     = 0b10000000 = 0b10000000
      // n & _ = 0b10000000 = 0b00000000
      while (self.peek()? & 0b10000000) != 0 {
        let num = (self.next()? ^ (0b10000000)) as $t; // If leftmost bit is 1, we drop it.

        // buf =      00000000_00000000_10000000_00000000
        // num =      00000000_00000000_00000000_00000001
        // num << 7 = 00000000_00000000_00000000_10000000
        // buf ^ num  00000000_00000000_10000000_10000000
        buf = buf ^ (num << shift);
        shift += 7;
      }
      let num = (self.next()?) as $t;
      buf = buf ^ (num << shift);

      let (msb_one, overflowed) = (1 as $buf_size).overflowing_shl(shift + 6);
      if overflowed {
        return Err(Trap::BitshiftOverflow)
      }
      if buf & (msb_one as $t) != 0 {
        Ok(-((1 << (shift + 7)) - buf))
      } else {
        Ok(buf)
      }
    }
  };
}

macro_rules! decode_float {
  ($ty: ty, $buf_ty: ty, $fn_name: ident, $convert: path, $bitwidth: expr) => {
    fn $fn_name(&mut self) -> Result<$ty> {
      let mut buf: $buf_ty = 0;
      let mut shift = 0;
      for _ in 0..($bitwidth / 8) {
        let num = self.next()? as $buf_ty;
        buf = buf ^ (num << shift);
        shift += 8;
      }
      Ok($convert(buf))
    }
  };
}

#[derive(Debug, PartialEq)]
pub struct Byte {
  bytes: Vec<u8>,
  pub bytes_decoded: Vec<Code>,
  byte_ptr: usize,
}

impl Byte {
  leb128!(i32, u32, decode_leb128_i32);
  leb128!(i64, u64, decode_leb128_i64);
  decode_float!(f32, u32, decode_f32, f32::from_bits, 32);
  decode_float!(f64, u64, decode_f64, f64::from_bits, 64);

  // FIXME: Generalize with macro decoding signed integer.
  fn decode_leb128_u32(&mut self) -> Result<u32> {
    let mut buf: u32 = 0;
    let mut shift = 0;
    while (self.peek()? & 0b10000000) != 0 {
      let num = (self.next()? ^ (0b10000000)) as u32;
      buf = buf ^ (num << shift);
      shift += 7;
    }
    let num = (self.next()?) as u32;
    buf = buf ^ (num << shift);
    Ok(buf)
  }

  pub fn new(bytes: Vec<u8>) -> Self {
    let (_, bytes) = bytes.split_at(8);
    Byte {
      bytes: bytes.to_vec(),
      bytes_decoded: vec![],
      byte_ptr: 0,
    }
  }

  fn has_next(&self) -> bool {
    self.byte_ptr < self.bytes.len()
  }

  fn peek(&self) -> Option<u8> {
    self.bytes.get(self.byte_ptr).map(|&x| x)
  }

  fn next(&mut self) -> Option<u8> {
    let el = self.bytes.get(self.byte_ptr);
    self.byte_ptr += 1;
    el.map(|&x| x)
  }

  fn decode_vec<T, F>(count_of_elements: u32, mut f: F) -> Result<Vec<T>>
  where
    F: FnMut() -> Result<T>,
  {
    let mut buf = vec![];
    for _ in 0..count_of_elements {
      let element = f()?;
      buf.push(element);
    }
    Ok(buf)
  }

  fn decode_section_type(&mut self) -> Result<Vec<FunctionType>> {
    let _bin_size_of_section = self.decode_leb128_i32()?;
    let count_of_type = self.decode_leb128_u32()?;
    Byte::decode_vec(count_of_type, || {
      let mut parameters = vec![];
      let mut returns = vec![];
      let _type_function = Code::from(self.next());
      let size_of_arity = self.decode_leb128_i32()?;
      for _ in 0..size_of_arity {
        parameters.push(ValueTypes::from(self.next()));
      }
      let size_of_result = self.decode_leb128_i32()?;
      for _ in 0..size_of_result {
        returns.push(ValueTypes::from(self.next()));
      }
      Ok(FunctionType::new(parameters, returns))
    })
  }

  fn decode_section_export(&mut self) -> Result<Vec<(String, usize)>> {
    let _bin_size_of_section = self.decode_leb128_i32()?;
    let count_of_exports = self.decode_leb128_u32()?;
    Byte::decode_vec(count_of_exports, || {
      let size_of_name = self.decode_leb128_u32()?;
      let mut buf = vec![];
      for _ in 0..size_of_name {
        buf.push(self.next()?);
      }
      let key = String::from_utf8(buf).expect("To encode export name has been failured.");
      let idx_of_fn = match ExportDescriptionCode::from(self.next()) {
        ExportDescriptionCode::ExportDescFunctionIdx => self.next()?,
        x => unimplemented!("{:?}", x),
      };
      Ok((key, idx_of_fn as usize))
    })
  }

  fn decode_memory_inst(&mut self) -> Result<(u32, u32)> {
    let align = self.decode_leb128_u32();
    let offset = self.decode_leb128_u32();
    match (align, offset) {
      (Ok(align), Ok(offset)) => Ok((align as u32, offset as u32)),
      (Err(Trap::BitshiftOverflow), _) | (_, Err(Trap::BitshiftOverflow)) => {
        println!("Decode int overflow");
        Err(Trap::MemoryAccessOutOfBounds)
      }
      _ => Err(Trap::Unknown),
    }
  }

  fn decode_section_code_internal(&mut self) -> Result<Vec<Inst>> {
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
          let tables = Byte::decode_vec(len, || self.decode_leb128_u32())?;
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
        Code::MemorySize => expressions.push(Inst::MemorySize),
        Code::MemoryGrow => expressions.push(Inst::MemoryGrow),
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

  fn decode_section_code(&mut self) -> Result<Vec<Result<(Vec<Inst>, Vec<ValueTypes>)>>> {
    let _bin_size_of_section = self.decode_leb128_u32()?;
    let count_of_code = self.decode_leb128_u32()?;
    Byte::decode_vec(count_of_code, || {
      let size_of_function = self.decode_leb128_u32()?;
      let end_of_function = self.byte_ptr + (size_of_function as usize);
      let count_of_locals = self.decode_leb128_u32()? as usize;
      let mut locals: Vec<ValueTypes> = Vec::with_capacity(count_of_locals);
      for _ in 0..count_of_locals {
        let count_of_type = self.decode_leb128_u32()?;
        let value_type = ValueTypes::from(self.next());
        for _ in 0..count_of_type {
          locals.push(value_type.clone());
        }
      }
      Ok(match self.decode_section_code_internal() {
        Ok(expressions) => Ok((expressions, locals)),
        Err(err) => {
          self.byte_ptr = end_of_function;
          Err(err)
        }
      })
    })
  }

  fn decode_section_function(&mut self) -> Result<Vec<u32>> {
    let _bin_size_of_section = self.decode_leb128_i32()?;
    let count_of_type_idx = self.decode_leb128_u32()?;
    Byte::decode_vec(count_of_type_idx, || {
      let idx = self.next()? as u32;
      Ok(idx)
    })
  }

  fn decode_limit(&mut self) -> Result<Limit> {
    match self.next() {
      Some(0x0) => {
        let min = self.decode_leb128_i32()?;
        Ok(Limit::NoUpperLimit(min as u32))
      }
      Some(0x1) => {
        let min = self.decode_leb128_i32()?;
        let max = self.decode_leb128_i32()?;
        Ok(Limit::HasUpperLimit(min as u32, max as u32))
      }
      x => unreachable!("Expected limit code, got {:?}", x),
    }
  }

  fn decode_section_memory(&mut self) -> Result<Vec<Limit>> {
    let _bin_size_of_section = self.decode_leb128_i32()?;
    let count_of_memory = self.decode_leb128_u32()?;
    Byte::decode_vec(count_of_memory, || self.decode_limit())
  }

  fn decode_section_data(&mut self) -> Result<Vec<Data>> {
    let _bin_size_of_section = self.decode_leb128_i32()?;
    let count_of_data = self.decode_leb128_u32()?;
    Byte::decode_vec(count_of_data, || {
      let memidx = self.decode_leb128_i32()? as u32;
      let offset = self.decode_section_code_internal()?;
      let mut size_of_data = self.next()?;
      let mut init = vec![];
      while size_of_data != 0 {
        size_of_data -= 1;
        init.push(self.next()?);
      }
      Ok(Data::new(memidx, offset, init))
    })
  }

  fn decode_section_table(&mut self) -> Result<Vec<TableType>> {
    let _bin_size_of_section = self.decode_leb128_i32()?;
    let count_of_data = self.decode_leb128_u32()?;
    Byte::decode_vec(count_of_data, || {
      let element_type = ElementType::from(self.next());
      let limit = self.decode_limit()?;
      Ok(TableType::new(element_type, limit))
    })
  }

  fn decode_section_global(&mut self) -> Result<Vec<GlobalInstance>> {
    let _bin_size_of_section = self.decode_leb128_i32()?;
    let count = self.decode_leb128_u32()?;
    Byte::decode_vec(count, || {
      let value_type = ValueTypes::from(self.next());
      let global_type = GlobalType::new(self.next(), value_type);
      let init = self.decode_section_code_internal()?;
      let value = init.first()?.get_value_ext();
      Ok(GlobalInstance::new(global_type, value))
    })
  }
  fn decode_function_idx(&mut self) -> Result<Vec<u32>> {
    let count = self.decode_leb128_u32()?;
    Byte::decode_vec(count, || self.decode_leb128_u32())
  }
  fn decode_section_element(&mut self) -> Result<Vec<Element>> {
    let _bin_size_of_section = self.decode_leb128_i32()?;
    let count = self.decode_leb128_u32()?;
    Byte::decode_vec(count, || {
      let table_idx = self.decode_leb128_u32()?;
      let offset = self.decode_section_code_internal()?;
      let init = self.decode_function_idx()?;
      Ok(Element::new(
        table_idx,
        Instructions::new(offset, vec![], vec![]),
        init,
      ))
    })
  }

  pub fn decode(&mut self) -> Result<Store> {
    let mut function_types = vec![];
    let mut index_of_types = vec![];
    let mut function_key_and_indexes = vec![];
    let mut list_of_expressions = vec![];
    let mut memories = vec![];
    let mut data = vec![];
    let mut table_types = vec![];
    let mut global_instances = vec![];
    let mut elements = vec![];
    while self.has_next() {
      let code = SectionCode::from(self.next());
      match code {
        SectionCode::Type => function_types = self.decode_section_type()?,
        SectionCode::Function => index_of_types = self.decode_section_function()?,
        SectionCode::Export => function_key_and_indexes = self.decode_section_export()?,
        SectionCode::Code => list_of_expressions = self.decode_section_code()?,
        SectionCode::Data => data = self.decode_section_data()?,
        SectionCode::Memory => memories = self.decode_section_memory()?,
        SectionCode::Table => table_types = self.decode_section_table()?,
        SectionCode::Global => global_instances = self.decode_section_global()?,
        SectionCode::Element => elements = self.decode_section_element()?,
        SectionCode::Custom | SectionCode::Import | SectionCode::Start => {
          unimplemented!("{:?}", code);
        }
      };
    }
    let mut function_instances = Vec::with_capacity(list_of_expressions.len());
    let memory_instances = data
      .into_iter()
      .map(|d| MemoryInstance::new(d, &memories))
      .collect::<Vec<_>>();

    let table_instances = elements
      .iter()
      .map(|el| {
        let table_type = table_types
          .get(el.table_idx as usize)
          .expect("Table type not found.");
        TableInstance::new(&table_type, el)
      })
      .collect();

    for idx_of_fn in 0..list_of_expressions.len() {
      let export_name = function_key_and_indexes
        .iter()
        .find(|(_, idx)| idx == &idx_of_fn)
        .map(|(key, _)| key.to_owned());
      let &index_of_type = index_of_types.get(idx_of_fn)?;
      let function_type = function_types.get(index_of_type as usize)?;
      let fnins = match list_of_expressions.get(idx_of_fn)? {
        Ok((expression, locals)) => FunctionInstance::new(
          export_name,
          Ok(function_type.to_owned()),
          locals.to_owned(),
          index_of_type,
          expression.to_owned(),
        ),
        Err(err) => FunctionInstance::new(
          export_name,
          Err(err.to_owned()),
          vec![],
          index_of_type,
          vec![],
        ),
      };
      function_instances.push(fnins);
    }
    Ok(Store::new(
      function_instances,
      memory_instances,
      table_instances,
      global_instances,
    ))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs::File;
  use std::io::Read;

  macro_rules! test_decode {
    ($fn_name:ident, $file_name:expr, $fn_insts: expr) => {
      #[test]
      fn $fn_name() {
        use self::Inst::*;
        let mut file = File::open(format!("./{}.wasm", $file_name)).unwrap();
        let mut buffer = vec![];
        let _ = file.read_to_end(&mut buffer);
        let mut bc = Byte::new(buffer);
        assert_eq!(bc.decode().unwrap().get_function_instance(), $fn_insts);
      }
    };
  }

  test_decode!(
    decode_cons8,
    "dist/cons8",
    vec![FunctionInstance::new(
      Some("_subject".to_owned()),
      Ok(FunctionType::new(vec![], vec![ValueTypes::I32],)),
      vec![],
      0,
      vec![I32Const(42), End],
    )]
  );
  test_decode!(
    decode_cons16,
    "dist/cons16",
    vec![FunctionInstance::new(
      Some("_subject".to_owned()),
      Ok(FunctionType::new(vec![], vec![ValueTypes::I32],)),
      vec![],
      0,
      vec![I32Const(255), End],
    )]
  );
  test_decode!(
    decode_signed,
    "dist/signed",
    vec![FunctionInstance::new(
      Some("_subject".to_owned()),
      Ok(FunctionType::new(vec![], vec![ValueTypes::I32],)),
      vec![],
      0,
      vec![I32Const(-129), End],
    )]
  );
  test_decode!(
    decode_add,
    "dist/add",
    vec![FunctionInstance::new(
      Some("_subject".to_owned()),
      Ok(FunctionType::new(
        vec![ValueTypes::I32, ValueTypes::I32],
        vec![ValueTypes::I32],
      )),
      vec![],
      0,
      vec![GetLocal(1), GetLocal(0), I32Add, End],
    )]
  );
  test_decode!(
    decode_sub,
    "dist/sub",
    vec![FunctionInstance::new(
      Some("_subject".to_owned()),
      Ok(FunctionType::new(
        vec![ValueTypes::I32],
        vec![ValueTypes::I32],
      )),
      vec![],
      0,
      vec![I32Const(100), GetLocal(0), I32Sub, End],
    )]
  );
  test_decode!(
    decode_add_five,
    "dist/add_five",
    vec![FunctionInstance::new(
      Some("_subject".to_owned()),
      Ok(FunctionType::new(
        vec![ValueTypes::I32, ValueTypes::I32],
        vec![ValueTypes::I32],
      )),
      vec![],
      0,
      vec![GetLocal(0), I32Const(10), I32Add, GetLocal(1), I32Add, End],
    )]
  );

  test_decode!(
    decode_if_lt,
    "dist/if_lt",
    vec![FunctionInstance::new(
      Some("_subject".to_owned()),
      Ok(FunctionType::new(
        vec![ValueTypes::I32],
        vec![ValueTypes::I32],
      )),
      vec![ValueTypes::I32],
      0,
      vec![
        GetLocal(0),
        I32Const(10),
        LessThanSign,
        If(6, 14),
        RuntimeValue(ValueTypes::I32),
        GetLocal(0),
        I32Const(10),
        I32Add,
        Else,
        GetLocal(0),
        I32Const(15),
        I32Add,
        SetLocal(1),
        GetLocal(0),
        I32Const(10),
        Equal,
        If(4, 2),
        RuntimeValue(ValueTypes::I32),
        I32Const(15),
        Else,
        GetLocal(1),
        End,
        End,
        End,
      ],
    )]
  );
  test_decode!(
    decode_if_gt,
    "dist/if_gt",
    vec![FunctionInstance::new(
      Some("_subject".to_owned()),
      Ok(FunctionType::new(
        vec![ValueTypes::I32],
        vec![ValueTypes::I32],
      )),
      vec![ValueTypes::I32],
      0,
      vec![
        GetLocal(0),
        I32Const(10),
        I32GreaterThanSign,
        If(6, 14),
        RuntimeValue(ValueTypes::I32),
        GetLocal(0),
        I32Const(10),
        I32Add,
        Else,
        GetLocal(0),
        I32Const(15),
        I32Add,
        SetLocal(1),
        GetLocal(0),
        I32Const(10),
        Equal,
        If(4, 2),
        RuntimeValue(ValueTypes::I32),
        I32Const(15),
        Else,
        GetLocal(1),
        End,
        End,
        End,
      ],
    )]
  );
  test_decode!(
    decode_if_eq,
    "dist/if_eq",
    vec![FunctionInstance::new(
      Some("_subject".to_owned()),
      Ok(FunctionType::new(
        vec![ValueTypes::I32],
        vec![ValueTypes::I32],
      )),
      vec![],
      0,
      vec![
        GetLocal(0),
        I32Const(10),
        Equal,
        If(4, 2),
        RuntimeValue(ValueTypes::I32),
        I32Const(5),
        Else,
        I32Const(10),
        End,
        GetLocal(0),
        I32Add,
        End,
      ],
    )]
  );
  test_decode!(
    decode_count,
    "dist/count",
    vec![FunctionInstance::new(
      Some("_subject".to_owned()),
      Ok(FunctionType::new(
        vec![ValueTypes::I32],
        vec![ValueTypes::I32],
      )),
      vec![ValueTypes::I32],
      0,
      vec![
        GetLocal(0),
        I32Const(0),
        I32LessEqualSign,
        If(5, 0),
        RuntimeValue(ValueTypes::Empty),
        I32Const(0),
        Return,
        End,
        GetLocal(0),
        I32Const(-1),
        I32Add,
        TeeLocal(1),
        GetLocal(0),
        I32Const(1),
        I32Add,
        I32Mul,
        GetLocal(0),
        I32Add,
        GetLocal(1),
        I64ExtendUnsignI32,
        GetLocal(0),
        I32Const(-2),
        I32Add,
        I64ExtendUnsignI32,
        I64Mul,
        I64Const(8589934591),
        I64And,
        I64Const(1),
        I64ShiftRightUnsign,
        I32WrapI64,
        I32Add,
        End,
      ],
    )]
  );
}
