use std::convert::From;
// /* BitAndAssign, , BitOrAssign, , BitXorAssign, */
use std::ops::{BitAnd, BitOr, BitXor};

#[derive(Debug, PartialEq, Clone)]
pub enum Op {
  I32Const(i32),
  I64Const(i64),
  GetLocal(usize),
  SetLocal(usize),
  TeeLocal(usize),
  I32CountLeadingZero,
  I32CountTrailingZero,
  I32CountNonZero,
  I32Add,
  I32Sub,
  I32Mul,
  I32DivSign,
  I32DivUnsign,
  I32RemSign,
  I32RemUnsign,
  I32And,
  I32Or,
  I32Xor,
  I32ShiftLeft,
  I32ShiftRIghtSign,
  I32ShiftRightUnsign,
  I32RotateLeft,
  I32RotateRight,
  Call(usize),
  I32EqualZero,
  Equal,
  NotEqual,
  LessThanSign,
  LessThanEqualSign,
  LessThanUnsign,
  GreaterThanSign,
  GreaterThanUnsign,
  I32GreaterThanUnsign,
  I32LessEqualSign,
  I32LessEqualUnsign,
  I32GreaterEqualSign,
  I32GreaterEqualUnsign,
  If(Vec<Op>, Vec<Op>),
  Select,
  Return,
  TypeI32,
  TypeEmpty,
  I64ExtendUnsignI32,
  I64Mul,
  I64And,
  I64ShiftRightUnsign,
  I32WrapI64,
}

pub enum Trap {
  DivisionOverflow,
  DivisionByZero,
}

#[derive(Debug, PartialEq, Clone)]
struct FunctionType {
  parameters: Vec<ValueTypes>,
  returns: Vec<ValueTypes>,
}

#[derive(Debug, PartialEq)]
pub struct FunctionInstance {
  export_name: Option<String>,
  function_type: FunctionType,
  locals: Vec<ValueTypes>,
  type_idex: u32,
  body: Vec<Op>,
}

impl FunctionInstance {
  pub fn call(&self) -> (Vec<Op>, Vec<ValueTypes>) {
    (self.body.to_owned(), self.locals.to_owned())
  }

  pub fn find(&self, key: &str) -> bool {
    // FIXME: When using function_index, we might get exported function by O(1).
    match &self.export_name {
      Some(name) => name.as_str() == key,
      _ => false,
    }
  }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ValueTypes {
  Empty,
  I32,
  // I64,
  // F32,
  // F64,
}

impl ValueTypes {
  fn from_byte(code: Option<u8>) -> Self {
    use self::ValueTypes::*;
    match code {
      Some(0x7f) => I32,
      Some(x) => unimplemented!("ValueTypes of {} does not implemented yet.", x),
      _ => unreachable!(),
    }
  }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Values {
  I32(i32),
  I64(i64),
  // F32,
  // F64,
}

macro_rules! numeric_instrunction {
  ($fn_name: ident,$op: ident) => {
    pub fn $fn_name(&self, other: &Self) -> Self {
      match (self, other) {
        (Values::I32(l), Values::I32(r)) => Values::I32(l.$op(*r)),
        (Values::I64(l), Values::I64(r)) => Values::I64(l.$op(*r)),
        _ => unimplemented!(),
      }
    }
  };
}

macro_rules! conditional_instrunction {
  ($fn_name: ident,$op: ident) => {
    pub fn $fn_name(&self, other: &Self) -> Self {
      match (self, other) {
        (Values::I32(l), Values::I32(r)) => Values::I32(if l.$op(r) { 1 } else { 0 }),
        (Values::I64(l), Values::I64(r)) => Values::I64(if l.$op(r) { 1 } else { 0 }),
        _ => unimplemented!(),
      }
    }
  };
}

impl Values {
  conditional_instrunction!(less_than, lt);
  conditional_instrunction!(less_than_equal, le);
  conditional_instrunction!(greater_than, gt);
  conditional_instrunction!(greater_than_equal, ge);
  conditional_instrunction!(equal, eq);
  conditional_instrunction!(not_equal, ne);

  numeric_instrunction!(and, bitand);
  numeric_instrunction!(or, bitor);
  numeric_instrunction!(xor, bitxor);
  numeric_instrunction!(add, wrapping_add);
  numeric_instrunction!(sub, wrapping_sub);
  numeric_instrunction!(mul, wrapping_mul);

  pub fn rem_s(&self, other: &Self) -> Result<Self, Trap> {
    match (self, other) {
      (Values::I32(l), Values::I32(r)) => {
        if *r == 0 {
          return Err(Trap::DivisionByZero);
        }
        let (divined, overflowed) = l.overflowing_rem(*r);
        if overflowed {
          Err(Trap::DivisionOverflow)
        } else {
          Ok(Values::I32(divined))
        }
      }
      // (Values::I64(l), Values::I64(r)) => Values::I64(l.overflowing_div(*r).0),
      _ => unimplemented!(),
    }
  }

  pub fn rem_u(&self, other: &Self) -> Result<Self, Trap> {
    match (self, other) {
      (Values::I32(l), Values::I32(r)) => {
        if *r == 0 {
          return Err(Trap::DivisionByZero);
        }
        let (divined, overflowed) = (*l as u32).overflowing_rem(*r as u32);
        if overflowed {
          Err(Trap::DivisionOverflow)
        } else {
          Ok(Values::I32(divined as i32))
        }
      }
      // (Values::I64(l), Values::I64(r)) => Values::I64(l.overflowing_div(*r).0),
      _ => unimplemented!(),
    }
  }

  pub fn div_u(&self, other: &Self) -> Result<Self, Trap> {
    match (self, other) {
      (Values::I32(l), Values::I32(r)) => {
        if *r == 0 {
          return Err(Trap::DivisionByZero);
        }
        let (divined, overflowed) = (*l as u32).overflowing_div(*r as u32);
        if overflowed {
          Err(Trap::DivisionOverflow)
        } else {
          Ok(Values::I32(divined as i32))
        }
      }
      // (Values::I64(l), Values::I64(r)) => Values::I64(l.overflowing_div(*r).0),
      _ => unimplemented!(),
    }
  }

  pub fn div_s(&self, other: &Self) -> Result<Self, Trap> {
    match (self, other) {
      (Values::I32(l), Values::I32(r)) => {
        if *r == 0 {
          return Err(Trap::DivisionByZero);
        }
        let (divined, overflowed) = l.overflowing_div(*r);
        if overflowed {
          Err(Trap::DivisionOverflow)
        } else {
          Ok(Values::I32(divined))
        }
      }
      // (Values::I64(l), Values::I64(r)) => Values::I64(l.overflowing_div(*r).0),
      _ => unimplemented!(),
    }
  }

  pub fn shift_left(&self, other: &Self) -> Self {
    match (self, other) {
      (Values::I32(i1), Values::I32(i2)) => {
        let shifted = i1.wrapping_shl(*i2 as u32);
        Values::I32(shifted)
      }
      _ => unimplemented!(),
    }
  }

  pub fn shift_right_sign(&self, other: &Self) -> Self {
    match (self, other) {
      (Values::I32(i1), Values::I32(i2)) => {
        let shifted = i1.wrapping_shr(*i2 as u32);
        Values::I32((shifted as u32) as i32)
      }
      _ => unimplemented!(),
    }
  }

  pub fn shift_right_unsign(&self, other: &Self) -> Self {
    match (self, other) {
      (Values::I32(i1), Values::I32(i2)) => {
        let i1 = *i1 as u32;
        let i2 = *i2 as u32;
        let shifted = i1.wrapping_shr(i2) as i32;
        let leftmost = 1 << 31;
        println!("{:b}", 1i32 << 31);
        println!("{:b}", shifted);
        Values::I32(if shifted & leftmost == 0 {
          shifted
        } else {
          // println!("{:b}", 1i32 << 31);
          // println!("{:b}", 1u32 << 31);
          println!("{}", shifted ^ leftmost);
          shifted ^ leftmost
        })
      }
      (Values::I64(i1), Values::I64(i2)) => {
        let shift = *i2 % 64;
        let result = i1 >> shift;
        Values::I64(result)
      }
      _ => unimplemented!(),
    }
  }

  pub fn is_truthy(&self) -> bool {
    match &self {
      Values::I32(n) => *n > 0,
      _ => unimplemented!(),
    }
  }
  pub fn extend_to_i64(&self) -> Self {
    match self {
      Values::I32(l) => Values::I64(i64::from(*l)),
      _ => unimplemented!(),
    }
  }
}

#[derive(Debug, PartialEq, Clone)]
enum SecionCode {
  SectionType,
  SectionFunction,
  SectionExport,
  SectionCode,
}

impl SecionCode {
  fn from_byte(code: Option<u8>) -> Self {
    use self::SecionCode::*;

    match code {
      Some(0x1) => SectionType,
      Some(0x3) => SectionFunction,
      Some(0x7) => SectionExport,
      Some(0xa) => SectionCode,
      x => unreachable!("SectionCode {:x?} does not supported yet.", x),
    }
  }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Code {
  ConstI32,
  ConstI64,

  ValueType(ValueTypes), // TODO: Conside to align 8bit
  TypeFunction,

  GetLocal,
  TeeLocal,
  SetLocal,
  I32CountLeadingZero,
  I32CountTrailingZero,
  I32CountNonZero,
  I32Add,
  I32Sub,
  I32Mul,
  I32WrapI64,
  I32DivSign,
  I32DivUnsign,
  I32RemSign,
  I32RemUnsign,
  I32And,
  I32Or,
  I32Xor,
  I32ShiftLeft,
  I32ShiftRIghtSign,
  I32ShiftRightUnsign,
  I32RotateLeft,
  I32RotateRight,

  I64ExtendUnsignI32,
  I64Mul,
  I64And,
  I64ShiftRightUnsign,

  ExportDescFunctionIdx,
  ExportDescTableIdx,
  ExportDescMemIdx,
  ExportDescGlobalIdx,

  Call,
  Select,

  I32EqualZero,
  Equal,
  NotEqual,
  LessThanSign,
  LessThanUnsign,
  LessThanEqualSign,
  GreaterThanSign,
  I32GreaterThanUnsign,
  I32LessEqualSign,
  I32LessEqualUnsign,
  I32GreaterEqualSign,
  I32GreaterEqualUnsign,

  If,
  Else,
  Return,
  End,
}

impl Code {
  fn from_byte(code: Option<u8>) -> Self {
    use self::Code::*;

    match code {
      Some(0x4) => If,
      Some(0x5) => Else,
      Some(0x0b) => End,
      Some(0x0f) => Return,
      Some(0x10) => Call,
      Some(0x1b) => Select,
      Some(0x20) => GetLocal,
      Some(0x21) => SetLocal,
      Some(0x22) => TeeLocal,
      Some(0x40) => ValueType(ValueTypes::Empty),
      Some(0x41) => ConstI32,
      Some(0x42) => ConstI64,
      Some(0x45) => I32EqualZero,
      Some(0x46) => Equal,
      Some(0x47) => NotEqual,
      Some(0x48) => LessThanSign,
      Some(0x49) => LessThanUnsign,
      Some(0x4a) => GreaterThanSign,
      Some(0x4b) => I32GreaterThanUnsign,
      Some(0x4c) => I32LessEqualSign,
      Some(0x4d) => I32LessEqualUnsign,
      Some(0x4e) => I32GreaterEqualSign,
      Some(0x4f) => I32GreaterEqualUnsign,
      Some(0x60) => TypeFunction,
      Some(0x67) => I32CountLeadingZero,
      Some(0x68) => I32CountTrailingZero,
      Some(0x69) => I32CountNonZero,
      Some(0x6a) => I32Add,
      Some(0x6b) => I32Sub,
      Some(0x6c) => I32Mul,
      Some(0x6d) => I32DivSign,
      Some(0x6e) => I32DivUnsign,
      Some(0x6f) => I32RemSign,
      Some(0x70) => I32RemUnsign,
      Some(0x71) => I32And,
      Some(0x72) => I32Or,
      Some(0x73) => I32Xor,
      Some(0x74) => I32ShiftLeft,
      Some(0x75) => I32ShiftRIghtSign,
      Some(0x76) => I32ShiftRightUnsign,
      Some(0x77) => I32RotateLeft,
      Some(0x78) => I32RotateRight,
      Some(0x7e) => I64Mul,
      Some(0x7f) => ValueType(ValueTypes::I32),
      Some(0x83) => I64And,
      Some(0x88) => I64ShiftRightUnsign,
      Some(0xa7) => I32WrapI64,
      Some(0xad) => I64ExtendUnsignI32,
      x => unreachable!("Code {:x?} does not supported yet.", x),
    }
  }

  fn is_end_of_code(code: Option<u8>) -> bool {
    match code {
      // Some(0x4) | Some(0x5) | Some(0x0b) => true,
      Some(0x0b) => true,
      _ => false,
    }
  }

  fn from_byte_to_export_description(code: Option<u8>) -> Self {
    use self::Code::*;
    match code {
      Some(0x00) => ExportDescFunctionIdx,
      Some(0x01) => ExportDescTableIdx,
      Some(0x02) => ExportDescMemIdx,
      Some(0x03) => ExportDescGlobalIdx,
      x => unreachable!("Export description code {:x?} does not supported yet.", x),
    }
  }
}

macro_rules! leb128 {
    ($t:ty, $fn_name: ident) => {
      fn $fn_name(&mut self) -> Option<$t> {
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
        if buf & (1 << (shift + 6)) != 0 {
          Some(-((1 << (shift + 7)) - buf))
        } else {
          Some(buf)
        }
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
  pub fn new(bytes: Vec<u8>) -> Self {
    Byte {
      bytes,
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

  fn peek_before(&self) -> Option<u8> {
    self.bytes.get(self.byte_ptr - 1).map(|&x| x)
  }

  fn next(&mut self) -> Option<u8> {
    let el = self.bytes.get(self.byte_ptr);
    self.byte_ptr += 1;
    el.map(|&x| x)
  }

  leb128!(i32, decode_leb128_i32);
  leb128!(i64, decode_leb128_i64);

  fn decode_section_type(&mut self) -> Option<Vec<FunctionType>> {
    let _bin_size_of_section = self.decode_leb128_i32()?;
    let count_of_type = self.decode_leb128_i32()?;
    let mut function_types = vec![];
    for _ in 0..count_of_type {
      let mut parameters = vec![];
      let mut returns = vec![];
      let _type_function = Code::from_byte(self.next());
      let size_of_arity = self.decode_leb128_i32()?;
      for _ in 0..size_of_arity {
        parameters.push(ValueTypes::from_byte(self.next()));
      }
      let size_of_result = self.decode_leb128_i32()?;
      for _ in 0..size_of_result {
        returns.push(ValueTypes::from_byte(self.next()));
      }
      function_types.push(FunctionType {
        parameters,
        returns,
      })
    }
    Some(function_types)
  }

  fn decode_section_export(&mut self) -> Option<Vec<(String, usize)>> {
    let _bin_size_of_section = self.decode_leb128_i32()?;
    let count_of_exports = self.decode_leb128_i32()?;
    let mut exports = vec![];
    for _ in 0..count_of_exports {
      let size_of_name = self.decode_leb128_i32()?;
      let mut buf = vec![];
      for _ in 0..size_of_name {
        buf.push(self.next()?);
      }
      let key = String::from_utf8(buf).expect("To encode export name has been failured.");
      let idx_of_fn = match Code::from_byte_to_export_description(self.next()) {
        Code::ExportDescFunctionIdx => self.next()?,
        _ => unimplemented!(),
      };
      exports.push((key, idx_of_fn as usize));
    }
    Some(exports)
  }

  fn decode_section_code_internal(&mut self) -> Option<Vec<Op>> {
    let mut expressions = vec![];
    while !(Code::is_end_of_code(self.peek())) {
      match Code::from_byte(self.next()) {
        Code::ConstI32 => expressions.push(Op::I32Const(self.decode_leb128_i32()?)),
        Code::ConstI64 => expressions.push(Op::I64Const(self.decode_leb128_i64()?)),
        // NOTE: It might be need to decode as LEB128 integer, too.
        Code::GetLocal => expressions.push(Op::GetLocal(self.next()? as usize)),
        Code::SetLocal => expressions.push(Op::SetLocal(self.next()? as usize)),
        Code::TeeLocal => expressions.push(Op::TeeLocal(self.next()? as usize)),
        Code::I32CountLeadingZero => expressions.push(Op::I32CountLeadingZero),
        Code::I32CountTrailingZero => expressions.push(Op::I32CountTrailingZero),
        Code::I32CountNonZero => expressions.push(Op::I32CountNonZero),
        Code::I32Add => expressions.push(Op::I32Add),
        Code::I32Sub => expressions.push(Op::I32Sub),
        Code::I32Mul => expressions.push(Op::I32Mul),
        Code::I32DivSign => expressions.push(Op::I32DivSign),
        Code::I32DivUnsign => expressions.push(Op::I32DivUnsign),
        Code::I32RemSign => expressions.push(Op::I32RemSign),
        Code::I32RemUnsign => expressions.push(Op::I32RemUnsign),
        Code::I32And => expressions.push(Op::I32And),
        Code::I32Or => expressions.push(Op::I32Or),
        Code::I32Xor => expressions.push(Op::I32Xor),
        Code::I32ShiftLeft => expressions.push(Op::I32ShiftLeft),
        Code::I32ShiftRIghtSign => expressions.push(Op::I32ShiftRIghtSign),
        Code::I32ShiftRightUnsign => expressions.push(Op::I32ShiftRightUnsign),
        Code::I32RotateLeft => expressions.push(Op::I32RotateLeft),
        Code::I32RotateRight => expressions.push(Op::I32RotateRight),
        Code::I64And => expressions.push(Op::I64And),
        Code::I64Mul => expressions.push(Op::I64Mul),
        Code::I64ExtendUnsignI32 => expressions.push(Op::I64ExtendUnsignI32),
        Code::I64ShiftRightUnsign => expressions.push(Op::I64ShiftRightUnsign),
        Code::I32WrapI64 => expressions.push(Op::I32WrapI64),
        Code::Call => expressions.push(Op::Call(self.next()? as usize)),
        Code::I32EqualZero => expressions.push(Op::I32EqualZero),
        Code::Equal => expressions.push(Op::Equal),
        Code::NotEqual => expressions.push(Op::NotEqual),
        Code::LessThanSign => expressions.push(Op::LessThanSign),
        Code::LessThanEqualSign => expressions.push(Op::LessThanEqualSign),
        Code::LessThanUnsign => expressions.push(Op::LessThanUnsign),
        Code::GreaterThanSign => expressions.push(Op::GreaterThanSign),
        Code::I32GreaterThanUnsign => expressions.push(Op::I32GreaterThanUnsign),
        Code::I32LessEqualSign => expressions.push(Op::I32LessEqualSign),
        Code::I32LessEqualUnsign => expressions.push(Op::I32LessEqualUnsign),
        Code::I32GreaterEqualSign => expressions.push(Op::I32GreaterEqualSign),
        Code::I32GreaterEqualUnsign => expressions.push(Op::I32GreaterEqualUnsign),
        Code::Select => expressions.push(Op::Select),
        Code::If => {
          let if_insts = self.decode_section_code_internal()?;
          match Code::from_byte(self.peek_before()) {
            Code::Else => {
              let else_insts = self.decode_section_code_internal()?;
              expressions.push(Op::If(if_insts, else_insts));
            }
            _ => expressions.push(Op::If(if_insts, vec![])),
          }
        }
        Code::Else => {
          return Some(expressions);
        }
        Code::End => {
          return Some(expressions);
        }
        Code::Return => expressions.push(Op::Return),
        Code::ValueType(ValueTypes::I32) => expressions.push(Op::TypeI32),
        Code::ValueType(ValueTypes::Empty) => expressions.push(Op::TypeEmpty),
        x => unimplemented!(
          "Code {:x?} does not supported yet. Current expressions -> {:?}",
          x,
          expressions
        ),
      };
    }
    match Code::from_byte(self.peek()) {
      Code::Else => Some(expressions),
      _ => {
        self.next(); // Drop End code.
        Some(expressions)
      }
    }
  }

  fn decode_section_code(&mut self) -> Option<Vec<(Vec<Op>, Vec<ValueTypes>)>> {
    let _bin_size_of_section = self.decode_leb128_i32()?;
    let mut codes = vec![];
    let count_of_code = self.decode_leb128_i32()?;
    for _idx_of_fn in 0..count_of_code {
      let _size_of_function = self.decode_leb128_i32()?;
      let count_of_locals = self.decode_leb128_i32()? as usize;
      // FIXME:
      let mut locals: Vec<ValueTypes> = Vec::with_capacity(count_of_locals);
      for _ in 0..count_of_locals {
        let _idx = self.next(); // NOTE: Index of local varibale type?
        locals.push(ValueTypes::from_byte(self.next()));
      }
      let mut expressions = self.decode_section_code_internal()?;
      codes.push((expressions, locals));
    }
    Some(codes)
  }

  fn decode_section_function(&mut self) -> Option<Vec<u32>> {
    let _bin_size_of_section = self.decode_leb128_i32()?;
    let count_of_type_idx = self.decode_leb128_i32()?;
    let mut type_indexes = vec![];
    for _idx_of_fn in 0..count_of_type_idx {
      type_indexes.push(self.next()? as u32);
    }
    Some(type_indexes)
  }

  pub fn decode(&mut self) -> Option<Vec<FunctionInstance>> {
    let mut function_types = vec![];
    let mut index_of_types = vec![];
    let mut function_key_and_indexes = vec![];
    let mut list_of_expressions = vec![];
    while self.has_next() {
      match SecionCode::from_byte(self.next()) {
        SecionCode::SectionType => {
          function_types = self.decode_section_type()?;
        }
        SecionCode::SectionFunction => {
          index_of_types = self.decode_section_function()?;
        }
        SecionCode::SectionExport => {
          function_key_and_indexes = self.decode_section_export()?;
        }
        SecionCode::SectionCode => {
          list_of_expressions = self.decode_section_code()?;
        }
      };
    }
    let mut function_instances = Vec::with_capacity(list_of_expressions.len());

    for idx_of_fn in 0..list_of_expressions.len() {
      let export_name = function_key_and_indexes
        .iter()
        .find(|(_, idx)| idx == &idx_of_fn)
        .map(|(key, _)| key.to_owned());
      let &index_of_type = index_of_types.get(idx_of_fn)?;
      let function_type = function_types.get(index_of_type as usize)?;
      let (expression, locals) = list_of_expressions.get(idx_of_fn)?;
      let fnins = FunctionInstance {
        export_name,
        function_type: function_type.to_owned(),
        locals: locals.to_owned(),
        type_idex: index_of_type,
        body: expression.to_owned(),
      };
      function_instances.push(fnins);
    }
    Some(function_instances)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use utils::read_wasm;

  #[test]
  fn repl() {
    println!("{:b}", 8u8);
    println!("{:b}", 8u8 >> 2);
    println!("{:b}", 8u8 << 2);
  }

  macro_rules! test_decode {
    ($fn_name:ident, $file_name:expr, $fn_insts: expr) => {
      #[test]
      fn $fn_name() {
        use Op::*;
        let wasm = read_wasm(format!("./{}.wasm", $file_name)).unwrap();
        let mut bc = Byte::new(wasm);
        assert_eq!(bc.decode().unwrap(), $fn_insts);
      }
    };
  }

  test_decode!(
    decode_cons8,
    "dist/cons8",
    vec![FunctionInstance {
      export_name: Some("_subject".to_owned()),
      function_type: FunctionType {
        parameters: vec![],
        returns: vec![ValueTypes::I32],
      },
      locals: vec![],
      type_idex: 0,
      body: vec![I32Const(42)],
    }]
  );

  test_decode!(
    decode_cons16,
    "dist/cons16",
    vec![FunctionInstance {
      export_name: Some("_subject".to_owned()),
      function_type: FunctionType {
        parameters: vec![],
        returns: vec![ValueTypes::I32],
      },
      locals: vec![],
      type_idex: 0,
      body: vec![I32Const(255)],
    }]
  );

  test_decode!(
    decode_signed,
    "dist/signed",
    vec![FunctionInstance {
      export_name: Some("_subject".to_owned()),
      function_type: FunctionType {
        parameters: vec![],
        returns: vec![ValueTypes::I32],
      },
      locals: vec![],
      type_idex: 0,
      body: vec![I32Const(-129)],
    }]
  );

  test_decode!(
    decode_add,
    "dist/add",
    vec![FunctionInstance {
      export_name: Some("_subject".to_owned()),
      function_type: FunctionType {
        parameters: vec![ValueTypes::I32, ValueTypes::I32],
        returns: vec![ValueTypes::I32],
      },
      locals: vec![],
      type_idex: 0,
      body: vec![GetLocal(1), GetLocal(0), I32Add],
    }]
  );

  test_decode!(
    decode_sub,
    "dist/sub",
    vec![FunctionInstance {
      export_name: Some("_subject".to_owned()),
      function_type: FunctionType {
        parameters: vec![ValueTypes::I32],
        returns: vec![ValueTypes::I32],
      },
      locals: vec![],
      type_idex: 0,
      body: vec![I32Const(100), GetLocal(0), I32Sub],
    }]
  );

  test_decode!(
    decode_add_five,
    "dist/add_five",
    vec![FunctionInstance {
      export_name: Some("_subject".to_owned()),
      function_type: FunctionType {
        parameters: vec![ValueTypes::I32, ValueTypes::I32],
        returns: vec![ValueTypes::I32],
      },
      locals: vec![],
      type_idex: 0,
      body: vec![GetLocal(0), I32Const(10), I32Add, GetLocal(1), I32Add],
    }]
  );

  test_decode!(
    decode_if_lt,
    "dist/if_lt",
    vec![FunctionInstance {
      export_name: Some("_subject".to_owned()),
      function_type: FunctionType {
        parameters: vec![ValueTypes::I32],
        returns: vec![ValueTypes::I32],
      },
      locals: vec![ValueTypes::I32],
      type_idex: 0,
      body: vec![
        GetLocal(0),
        I32Const(10),
        LessThanSign,
        If(
          vec![TypeI32, GetLocal(0), I32Const(10), I32Add],
          vec![
            GetLocal(0),
            I32Const(15),
            I32Add,
            SetLocal(1),
            GetLocal(0),
            I32Const(10),
            Equal,
            If(vec![TypeI32, I32Const(15),], vec![GetLocal(1)]),
          ]
        ),
      ],
    }]
  );
  test_decode!(
    decode_if_gt,
    "dist/if_gt",
    vec![FunctionInstance {
      export_name: Some("_subject".to_owned()),
      function_type: FunctionType {
        parameters: vec![ValueTypes::I32],
        returns: vec![ValueTypes::I32],
      },
      locals: vec![ValueTypes::I32],
      type_idex: 0,
      body: vec![
        GetLocal(0),
        I32Const(10),
        GreaterThanSign,
        If(
          vec![TypeI32, GetLocal(0), I32Const(10), I32Add],
          vec![
            GetLocal(0),
            I32Const(15),
            I32Add,
            SetLocal(1),
            GetLocal(0),
            I32Const(10),
            Equal,
            If(vec![TypeI32, I32Const(15)], vec![GetLocal(1)]),
          ]
        ),
      ],
    }]
  );
  test_decode!(
    decode_if_eq,
    "dist/if_eq",
    vec![FunctionInstance {
      export_name: Some("_subject".to_owned()),
      function_type: FunctionType {
        parameters: vec![ValueTypes::I32],
        returns: vec![ValueTypes::I32],
      },
      locals: vec![],
      type_idex: 0,
      body: vec![
        GetLocal(0),
        I32Const(10),
        Equal,
        If(vec![TypeI32, I32Const(5)], vec![I32Const(10)]),
        GetLocal(0),
        I32Add,
      ],
    }]
  );
  test_decode!(
    decode_count,
    "dist/count",
    vec![FunctionInstance {
      export_name: Some("_subject".to_owned()),
      function_type: FunctionType {
        parameters: vec![ValueTypes::I32],
        returns: vec![ValueTypes::I32],
      },
      locals: vec![ValueTypes::I32],
      type_idex: 0,
      body: vec![
        GetLocal(0),
        I32Const(0),
        LessThanEqualSign,
        If(vec![TypeEmpty, I32Const(0), Return], vec![]),
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
      ],
    }]
  );
}
