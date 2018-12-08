use code::{SectionCode, ValueTypes};
use decode::decodable::Decodable;
use decode::*;
use element::Element;
use function::FunctionType;
use global::GlobalInstance;
use inst::Inst;
use memory::{Data, Limit};
use section::Section;
use std::convert::From;
use std::default::Default;
use store::Store;
use table::TableType;
use trap::Result;

#[derive(Debug, PartialEq)]
pub struct Byte {
  bytes: Vec<u8>,
  byte_ptr: usize,
}

impl Byte {
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

  fn decode_section(&mut self) -> Result<Vec<u8>> {
    let bin_size_of_section = self.decode_leb128_u32()?;
    let start = self.byte_ptr;
    let end = start + bin_size_of_section as usize;
    let bytes = self.bytes.drain(start..end).collect::<Vec<_>>();
    Ok(bytes)
  }

  fn decode_section_code(&mut self) -> Result<Vec<Result<(Vec<Inst>, Vec<ValueTypes>)>>> {
    sec_code::Section::new(self.decode_section()?).decode()
  }

  fn decode_section_type(&mut self) -> Result<Vec<FunctionType>> {
    sec_type::Section::new(self.decode_section()?).decode()
  }

  fn decode_section_export(&mut self) -> Result<Vec<(String, usize)>> {
    sec_export::Section::new(self.decode_section()?).decode()
  }

  fn decode_section_function(&mut self) -> Result<Vec<u32>> {
    sec_function::Section::new(self.decode_section()?).decode()
  }

  fn decode_section_memory(&mut self) -> Result<Vec<Limit>> {
    sec_memory::Section::new(self.decode_section()?).decode()
  }

  fn decode_section_data(&mut self) -> Result<Vec<Data>> {
    sec_data::Section::new(self.decode_section()?).decode()
  }

  fn decode_section_table(&mut self) -> Result<Vec<TableType>> {
    sec_table::Section::new(self.decode_section()?).decode()
  }

  fn decode_section_global(&mut self) -> Result<Vec<GlobalInstance>> {
    sec_global::Section::new(self.decode_section()?).decode()
  }
  fn decode_section_element(&mut self) -> Result<Vec<Element>> {
    sec_element::Section::new(self.decode_section()?).decode()
  }

  pub fn decode(&mut self) -> Result<Store> {
    let mut section: Section = Default::default();
    while self.has_next() {
      let code = SectionCode::from(self.next());
      match code {
        SectionCode::Type => section.function_types(self.decode_section_type()?),
        SectionCode::Function => section.functions(self.decode_section_function()?),
        SectionCode::Export => section.exports(self.decode_section_export()?),
        SectionCode::Code => section.codes(self.decode_section_code()?),
        SectionCode::Data => section.datas(self.decode_section_data()?),
        SectionCode::Memory => section.limits(self.decode_section_memory()?),
        SectionCode::Table => section.tables(self.decode_section_table()?),
        SectionCode::Global => section.globals(self.decode_section_global()?),
        SectionCode::Element => section.elements(self.decode_section_element()?),
        SectionCode::Custom | SectionCode::Import | SectionCode::Start => {
          unimplemented!("{:?}", code);
        }
      };
    }
    Ok(section.complete())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use code::ValueTypes;
  use function::{FunctionInstance, FunctionType};
  use inst::Inst;
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
