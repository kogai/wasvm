use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Op {
  Const(i32),
  GetLocal(usize),
  SetLocal(usize),
  Add,
  Call(usize),
}

#[derive(Debug, PartialEq, Clone)]
struct FunctionType {
  parameters: Vec<ValueTypes>,
  returns: Vec<ValueTypes>,
}

#[derive(Debug, PartialEq)]
pub struct FunctionInstance {
  function_type: FunctionType,
  locals: Vec<Values>,
  type_idex: u32,
  body: Vec<Op>,
}

impl FunctionInstance {
  pub fn call(&self) -> Vec<Op> {
    self.body.to_owned()
  }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ValueTypes {
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
  // I64,
  // F32,
  // F64,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Code {
  SectionType,
  SectionFunction,
  SectionExport,
  SectionCode,
  ConstI32,

  ValueType(ValueTypes),
  TypeFunction,

  GetLocal,
  SetLocal,
  Add,

  ExportDescFunctionIdx,
  ExportDescTableIdx,
  ExportDescMemIdx,
  ExportDescGlobalIdx,

  Call,
  End,
}

impl Code {
  fn from_byte(code: Option<u8>) -> Self {
    use self::Code::*;

    match code {
      Some(0x1) => SectionType,
      Some(0x3) => SectionFunction,
      Some(0x7) => SectionExport,
      Some(0xa) => SectionCode,
      Some(0x7f) => ValueType(ValueTypes::I32),
      Some(0x41) => ConstI32,
      Some(0x60) => TypeFunction,
      Some(0x20) => GetLocal,
      Some(0x21) => SetLocal,
      Some(0x6a) => Add,
      Some(0x10) => Call,
      Some(0x0b) => End,
      x => unreachable!("Code {:x?} does not supported yet.", x),
    }
  }

  fn is_end_of_code(code: Option<u8>) -> bool {
    match code {
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
      _ => unreachable!(),
    }
  }
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

  fn next(&mut self) -> Option<u8> {
    let el = self.bytes.get(self.byte_ptr);
    self.byte_ptr += 1;
    el.map(|&x| x)
  }

  fn decode_section_type(&mut self) -> Option<Vec<FunctionType>> {
    let _bin_size_of_section = self.next()?;
    let count_of_type = self.next()?;
    let mut function_types = vec![];
    for _ in 0..count_of_type {
      let mut parameters = vec![];
      let mut returns = vec![];
      let _type_function = Code::from_byte(self.next());
      let size_of_arity = self.next()?;
      for _ in 0..size_of_arity {
        parameters.push(ValueTypes::from_byte(self.next()));
      }
      let size_of_result = self.next()?;
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

  fn decode_section_export(&mut self) -> Option<Vec<(String, u8)>> {
    let _bin_size_of_section = self.next()?;
    let count_of_exports = self.next()?;
    let mut exports = vec![];
    for _ in 0..count_of_exports {
      let size_of_name = self.next()?;
      let mut buf = vec![];
      for _ in 0..size_of_name {
        buf.push(self.next()?);
      }
      let key = String::from_utf8(buf).expect("To encode export name has been failured.");
      let idx_of_fn = match Code::from_byte_to_export_description(self.next()) {
        Code::ExportDescFunctionIdx => self.next()?,
        _ => unimplemented!(),
      };
      exports.push((key, idx_of_fn));
    }
    Some(exports)
  }

  fn decode_section_code(&mut self) -> Option<Vec<Vec<Op>>> {
    let _bin_size_of_section = self.next()?;
    let mut codes = vec![];
    let count_of_code = self.next()?;
    for _idx_of_fn in 0..count_of_code {
      let mut expressions = vec![];
      let _size_of_function = self.next()?;
      let size_of_locals = self.next()? as usize;
      // FIXME:
      let mut locals: Vec<ValueTypes> = Vec::with_capacity(size_of_locals);
      for _ in 0..size_of_locals {
        let _ = self.next();
        locals.push(ValueTypes::from_byte(self.next()));
      }
      while !(Code::is_end_of_code(self.peek())) {
        match Code::from_byte(self.next()) {
          Code::ConstI32 => {
            let mut buf: i32 = 0;
            let mut shift = 0;
            while !(Code::is_end_of_code(self.peek())) {
              let n = self.next()?;
              // n     = 0b11111111
              // _     = 0b10000000
              // n & _ = 0b10000000
              let num = if (n & 0b10000000) != 0 {
                n ^ (0b10000000) // If leftmost bit is 1, we drop it.
              } else {
                n
              } as i32;
              // buf =      00000000_00000000_10000000_00000000
              // num =      00000000_00000000_00000000_00000001
              // num << 7 = 00000000_00000000_00000000_10000000
              // buf ^ num  00000000_00000000_10000000_10000000
              buf = buf ^ (num << shift);
              shift += 7;
            }
            expressions.push(Op::Const(buf));
          }
          Code::GetLocal => {
            // NOTE: It might be need to decode as LEB128 integer, too.
            expressions.push(Op::GetLocal(self.next()? as usize));
          }
          Code::SetLocal => {
            expressions.push(Op::SetLocal(self.next()? as usize));
          }
          Code::Add => {
            expressions.push(Op::Add);
          }
          Code::Call => {
            expressions.push(Op::Call(self.next()? as usize));
          }
          x => unimplemented!(
            "Code {:x?} does not supported yet. Current expressions -> {:?}",
            x,
            expressions
          ),
        };
      }
      self.next(); // Drop End code.
      codes.push(expressions);
    }
    Some(codes)
  }

  fn decode_section_function(&mut self) -> Option<Vec<u32>> {
    let _bin_size_of_section = self.next()?;
    let count_of_type_idx = self.next()?;
    let mut type_indexes = vec![];
    for _idx_of_fn in 0..count_of_type_idx {
      type_indexes.push(self.next()? as u32);
    }
    Some(type_indexes)
  }

  pub fn decode(&mut self) -> Option<HashMap<String, FunctionInstance>> {
    let mut function_instances = HashMap::new();
    let mut function_types = vec![];
    let mut index_of_types = vec![];
    let mut function_key_and_indexes = vec![];
    let mut list_of_expressions = vec![];
    while self.has_next() {
      match Code::from_byte(self.next()) {
        Code::SectionType => {
          function_types = self.decode_section_type()?;
        }
        Code::SectionFunction => {
          index_of_types = self.decode_section_function()?;
        }
        Code::SectionExport => {
          function_key_and_indexes = self.decode_section_export()?;
        }
        Code::SectionCode => {
          list_of_expressions = self.decode_section_code()?;
        }
        x => unreachable!("{:?}", x),
      };
    }
    println!("{:?}", list_of_expressions);
    for (key, idx_of_fn) in &function_key_and_indexes {
      let function_type = function_types.get(*idx_of_fn as usize)?;
      let locals: Vec<Values> = vec![];
      let &index_of_type = index_of_types.get(*idx_of_fn as usize)?;
      let expression = list_of_expressions.get(*idx_of_fn as usize)?;
      let fnins = FunctionInstance {
        function_type: function_type.to_owned(),
        locals,
        type_idex: index_of_type,
        body: expression.to_owned(),
      };
      function_instances.insert(key.to_owned(), fnins);
    }
    Some(function_instances)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::iter::FromIterator;
  use utils::read_wasm;

  macro_rules! test_decode {
    ($fn_name:ident, $file_name:expr, $fn_insts: expr) => {
      #[test]
      fn $fn_name() {
        use Op::*;
        let wasm = read_wasm(format!("./dist/{}.wasm", $file_name)).unwrap();
        let mut bc = Byte::new(wasm);
        assert_eq!(
          bc.decode().unwrap(),
          HashMap::from_iter($fn_insts.into_iter())
        );
      }
    };
  }

  test_decode!(
    decode_cons8,
    "cons8",
    vec![(
      "_subject".to_owned(),
      FunctionInstance {
        function_type: FunctionType {
          parameters: vec![],
          returns: vec![ValueTypes::I32],
        },
        locals: vec![],
        type_idex: 0,
        body: vec![Const(42)],
      }
    )]
  );

  test_decode!(
    decode_cons16,
    "cons16",
    vec![(
      "_subject".to_owned(),
      FunctionInstance {
        function_type: FunctionType {
          parameters: vec![],
          returns: vec![ValueTypes::I32],
        },
        locals: vec![],
        type_idex: 0,
        body: vec![Const(255)],
      }
    )]
  );

  test_decode!(
    decode_locals,
    "locals",
    vec![(
      "_subject".to_owned(),
      FunctionInstance {
        function_type: FunctionType {
          parameters: vec![ValueTypes::I32],
          returns: vec![ValueTypes::I32],
        },
        locals: vec![],
        type_idex: 0,
        body: vec![GetLocal(0)],
      }
    )]
  );

  test_decode!(
    decode_add,
    "add",
    vec![(
      "_subject".to_owned(),
      FunctionInstance {
        function_type: FunctionType {
          parameters: vec![ValueTypes::I32, ValueTypes::I32],
          returns: vec![ValueTypes::I32],
        },
        locals: vec![],
        type_idex: 0,
        body: vec![GetLocal(1), GetLocal(0), Add],
      }
    )]
  );

  test_decode!(
    decode_add_five,
    "add_five",
    vec![(
      "_subject".to_owned(),
      FunctionInstance {
        function_type: FunctionType {
          parameters: vec![ValueTypes::I32, ValueTypes::I32],
          returns: vec![ValueTypes::I32],
        },
        locals: vec![],
        type_idex: 1,
        body: vec![
          GetLocal(0),
          Call(0),
          SetLocal(2),
          GetLocal(1),
          Call(0),
          GetLocal(2),
          Add
        ],
      }
    )]
  );
}
