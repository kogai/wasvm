use code::{Code, ExportDescriptionCode, SectionCode, ValueTypes};
use inst::Inst;
use std::convert::From;
use trap::Trap;

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

#[derive(Debug)]
pub enum Memory {
  NoUpperLimit(u32),
  HasUpperLimit(u32, u32),
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

macro_rules! leb128 {
  ($t:ty, $buf_size: ty, $fn_name: ident) => {
    fn $fn_name(&mut self) -> Result<$t, Trap> {
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
        parameters.push(ValueTypes::from(self.next()));
      }
      let size_of_result = self.decode_leb128_i32()?;
      for _ in 0..size_of_result {
        returns.push(ValueTypes::from(self.next()));
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

  fn decode_memory_inst(&mut self) -> Option<(u32, u32)> {
    let align = self.decode_leb128_i32()? as u32;
    let offset = self.decode_leb128_i32()? as u32;
    Some((align, offset))
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

        Code::I32Load => {
          let (align, offset) = self.decode_memory_inst()?;
          expressions.push(Inst::I32Load(align, offset));
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
        Code::I64ExtendUnsignI32 => expressions.push(Inst::I64ExtendUnsignI32),

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
        Code::Call => expressions.push(Inst::Call(self.next()? as usize)),
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
        Code::Select => expressions.push(Inst::Select),
        Code::If => {
          let return_type = ValueTypes::from(self.next());
          let if_insts = self.decode_section_code_internal()?;
          match Code::from(self.peek_before()) {
            Code::Else => {
              let else_insts = self.decode_section_code_internal()?;
              expressions.push(Inst::If(return_type, if_insts, else_insts));
            }
            _ => expressions.push(Inst::If(return_type, if_insts, vec![])),
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
        locals.push(ValueTypes::from(self.next()));
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

  fn decode_section_memory(&mut self) -> Option<Vec<Memory>> {
    let _bin_size_of_section = self.decode_leb128_i32()?;
    let count_of_memory = self.decode_leb128_i32()?;
    let mut results = vec![];
    for _ in 0..count_of_memory {
      match self.next() {
        Some(0x0) => {
          let min = self.decode_leb128_i32()?;
          results.push(Memory::NoUpperLimit(min as u32))
        }
        Some(0x1) => {
          let min = self.decode_leb128_i32()?;
          let max = self.decode_leb128_i32()?;
          results.push(Memory::HasUpperLimit(min as u32, max as u32))
        }
        x => unreachable!("Expected limit of memory-type, got {:?}", x),
      }
    }
    Some(results)
  }

  pub fn decode(&mut self) -> Option<Vec<FunctionInstance>> {
    let mut function_types = vec![];
    let mut index_of_types = vec![];
    let mut function_key_and_indexes = vec![];
    let mut list_of_expressions = vec![];
    let mut memories = vec![];
    while self.has_next() {
      match SectionCode::from(self.next()) {
        SectionCode::Type => {
          function_types = self.decode_section_type()?;
        }
        SectionCode::Function => {
          index_of_types = self.decode_section_function()?;
        }
        SectionCode::Export => {
          function_key_and_indexes = self.decode_section_export()?;
        }
        SectionCode::Code => {
          list_of_expressions = self.decode_section_code()?;
        }
        SectionCode::Data => {
          unimplemented!();
        }
        SectionCode::Memory => {
          memories = self.decode_section_memory()?;
        }
        SectionCode::Custom
        | SectionCode::Import
        | SectionCode::Table
        | SectionCode::Global
        | SectionCode::Start
        | SectionCode::Element => {
          unimplemented!();
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
          ValueTypes::I32,
          vec![GetLocal(0), I32Const(10), I32Add],
          vec![
            GetLocal(0),
            I32Const(15),
            I32Add,
            SetLocal(1),
            GetLocal(0),
            I32Const(10),
            Equal,
            If(ValueTypes::I32, vec![I32Const(15),], vec![GetLocal(1)]),
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
        I32GreaterThanSign,
        If(
          ValueTypes::I32,
          vec![GetLocal(0), I32Const(10), I32Add],
          vec![
            GetLocal(0),
            I32Const(15),
            I32Add,
            SetLocal(1),
            GetLocal(0),
            I32Const(10),
            Equal,
            If(ValueTypes::I32, vec![I32Const(15)], vec![GetLocal(1)]),
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
        If(ValueTypes::I32, vec![I32Const(5)], vec![I32Const(10)]),
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
        If(ValueTypes::Empty, vec![I32Const(0), Return], vec![]),
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
