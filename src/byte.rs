use std::convert::From;
// /* BitAndAssign, , BitOrAssign, , BitXorAssign, */
use code::{Code, ExportDescriptionCode, SecionCode};
use inst::{Inst, Trap};
use std::ops::{BitAnd, BitOr, BitXor};

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
  body: Vec<Inst>,
}

impl FunctionInstance {
  pub fn call(&self) -> (Vec<Inst>, Vec<ValueTypes>) {
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

  pub fn equal_zero(&self) -> Self {
    match self {
      Values::I32(n) => Values::I32(if *n == 0 { 1 } else { 0 }),
      _ => unimplemented!(),
    }
  }

  pub fn less_than_unsign(&self, other: &Self) -> Self {
    match (self, other) {
      (Values::I32(l), Values::I32(r)) => {
        let l1 = *l as u32;
        let r1 = *r as u32;
        let result = l1.lt(&r1);
        Values::I32(if result { 1 } else { 0 })
      }
      _ => unimplemented!(),
    }
  }

  pub fn less_than_equal_unsign(&self, other: &Self) -> Self {
    match (self, other) {
      (Values::I32(l), Values::I32(r)) => {
        let l1 = *l as u32;
        let r1 = *r as u32;
        let result = l1.le(&r1);
        Values::I32(if result { 1 } else { 0 })
      }
      _ => unimplemented!(),
    }
  }

  pub fn greater_than_unsign(&self, other: &Self) -> Self {
    match (self, other) {
      (Values::I32(l), Values::I32(r)) => {
        let l1 = *l as u32;
        let r1 = *r as u32;
        let result = l1.gt(&r1);
        Values::I32(if result { 1 } else { 0 })
      }
      _ => unimplemented!(),
    }
  }

  pub fn greater_than_equal_unsign(&self, other: &Self) -> Self {
    match (self, other) {
      (Values::I32(l), Values::I32(r)) => {
        let l1 = *l as u32;
        let r1 = *r as u32;
        let result = l1.ge(&r1);
        Values::I32(if result { 1 } else { 0 })
      }
      _ => unimplemented!(),
    }
  }

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
        Values::I32(shifted)
      }
      (Values::I64(i1), Values::I64(i2)) => {
        let i1 = *i1 as u64;
        let i2 = *i2 as u64;
        let shifted = i1.wrapping_shr((i2 % 64) as u32) as i64;
        Values::I64(shifted)
      }
      _ => unimplemented!(),
    }
  }

  pub fn rotate_left(&self, other: &Self) -> Self {
    match (self, other) {
      (Values::I32(i1), Values::I32(i2)) => Values::I32(i1.rotate_left(*i2 as u32)),
      _ => unimplemented!(),
    }
  }

  pub fn rotate_right(&self, other: &Self) -> Self {
    match (self, other) {
      (Values::I32(i1), Values::I32(i2)) => Values::I32(i1.rotate_right(*i2 as u32)),
      _ => unimplemented!(),
    }
  }

  pub fn is_truthy(&self) -> bool {
    match &self {
      Values::I32(n) => *n > 0,
      _ => unimplemented!(),
    }
  }

  pub fn count_leading_zero(&self) -> Self {
    match self {
      Values::I32(l) => Values::I32(l.leading_zeros() as i32),
      _ => unimplemented!(),
    }
  }

  pub fn count_trailing_zero(&self) -> Self {
    match self {
      Values::I32(l) => Values::I32(l.trailing_zeros() as i32),
      _ => unimplemented!(),
    }
  }

  pub fn pop_count(&self) -> Self {
    match self {
      Values::I32(l) => Values::I32(l.count_ones() as i32),
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
      let _type_function = Code::from(self.next());
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
      let idx_of_fn = match ExportDescriptionCode::from(self.next()) {
        ExportDescriptionCode::ExportDescFunctionIdx => self.next()?,
        _ => unimplemented!(),
      };
      exports.push((key, idx_of_fn as usize));
    }
    Some(exports)
  }

  fn decode_section_code_internal(&mut self) -> Option<Vec<Inst>> {
    let mut expressions = vec![];
    while !(Code::is_end_of_code(self.peek())) {
      match Code::from(self.next()) {
        Code::ConstI32 => expressions.push(Inst::I32Const(self.decode_leb128_i32()?)),
        Code::ConstI64 => expressions.push(Inst::I64Const(self.decode_leb128_i64()?)),
        // NOTE: It might be need to decode as LEB128 integer, too.
        Code::GetLocal => expressions.push(Inst::GetLocal(self.next()? as usize)),
        Code::SetLocal => expressions.push(Inst::SetLocal(self.next()? as usize)),
        Code::TeeLocal => expressions.push(Inst::TeeLocal(self.next()? as usize)),
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
        Code::I64And => expressions.push(Inst::I64And),
        Code::I64Mul => expressions.push(Inst::I64Mul),
        Code::I64ExtendUnsignI32 => expressions.push(Inst::I64ExtendUnsignI32),
        Code::I64ShiftRightUnsign => expressions.push(Inst::I64ShiftRightUnsign),
        Code::I32WrapI64 => expressions.push(Inst::I32WrapI64),
        Code::Call => expressions.push(Inst::Call(self.next()? as usize)),
        Code::I32EqualZero => expressions.push(Inst::I32EqualZero),
        Code::Equal => expressions.push(Inst::Equal),
        Code::NotEqual => expressions.push(Inst::NotEqual),
        Code::LessThanSign => expressions.push(Inst::LessThanSign),
        Code::LessThanEqualSign => expressions.push(Inst::I32LessEqualSign),
        Code::LessThanUnsign => expressions.push(Inst::LessThanUnsign),
        Code::GreaterThanSign => expressions.push(Inst::GreaterThanSign),
        Code::I32GreaterThanUnsign => expressions.push(Inst::I32GreaterThanUnsign),
        Code::I32LessEqualSign => expressions.push(Inst::I32LessEqualSign),
        Code::I32LessEqualUnsign => expressions.push(Inst::I32LessEqualUnsign),
        Code::I32GreaterEqualSign => expressions.push(Inst::I32GreaterEqualSign),
        Code::I32GreaterEqualUnsign => expressions.push(Inst::I32GreaterEqualUnsign),
        Code::Select => expressions.push(Inst::Select),
        Code::If => {
          let if_insts = self.decode_section_code_internal()?;
          match Code::from(self.peek_before()) {
            Code::Else => {
              let else_insts = self.decode_section_code_internal()?;
              expressions.push(Inst::If(if_insts, else_insts));
            }
            _ => expressions.push(Inst::If(if_insts, vec![])),
          }
        }
        Code::Else => {
          return Some(expressions);
        }
        Code::End => {
          return Some(expressions);
        }
        Code::Return => expressions.push(Inst::Return),
        Code::TypeValueI32 => expressions.push(Inst::TypeI32),
        Code::TypeValueEmpty => expressions.push(Inst::TypeEmpty),
        x => unimplemented!(
          "Code {:x?} does not supported yet. Current expressions -> {:?}",
          x,
          expressions
        ),
      };
    }
    match Code::from(self.peek()) {
      Code::Else => Some(expressions),
      _ => {
        self.next(); // Drop End code.
        Some(expressions)
      }
    }
  }

  fn decode_section_code(&mut self) -> Option<Vec<(Vec<Inst>, Vec<ValueTypes>)>> {
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
      match SecionCode::from(self.next()) {
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
        use self::Inst::*;
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
        I32LessEqualSign,
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
