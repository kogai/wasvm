use code::SectionCode;
use decode::decodable::Decodable;
use decode::section::Section;
use decode::*;
use std::convert::From;
use std::default::Default;
use store::Store;
use trap::Result;

impl_decodable!(Byte);

impl Byte {
  pub fn new_with_drop(bytes: Vec<u8>) -> Self {
    let (_, bytes) = bytes.split_at(8);
    Byte::new(bytes.to_vec())
  }

  fn has_next(&self) -> bool {
    self.byte_ptr < self.bytes.len()
  }

  fn decode_section(&mut self) -> Result<Vec<u8>> {
    let bin_size_of_section = self.decode_leb128_u32()?;
    let start = self.byte_ptr;
    let end = start + bin_size_of_section as usize;
    let bytes = self.bytes.drain(start..end).collect::<Vec<_>>();
    Ok(bytes)
  }

  pub fn decode(&mut self) -> Result<Store> {
    let mut section: Section = Default::default();
    while self.has_next() {
      let code = SectionCode::from(self.next());
      match code {
        SectionCode::Type => {
          section.function_types(sec_type::Section::new(self.decode_section()?).decode()?)
        }
        SectionCode::Function => {
          section.functions(sec_function::Section::new(self.decode_section()?).decode()?)
        }
        SectionCode::Export => {
          section.exports(sec_export::Section::new(self.decode_section()?).decode()?)
        }
        SectionCode::Code => {
          section.codes(sec_code::Section::new(self.decode_section()?).decode()?)
        }
        SectionCode::Data => {
          section.datas(sec_data::Section::new(self.decode_section()?).decode()?)
        }
        SectionCode::Memory => {
          section.limits(sec_memory::Section::new(self.decode_section()?).decode()?)
        }
        SectionCode::Table => {
          section.tables(sec_table::Section::new(self.decode_section()?).decode()?)
        }
        SectionCode::Global => {
          section.globals(sec_global::Section::new(self.decode_section()?).decode()?)
        }
        SectionCode::Element => {
          section.elements(sec_element::Section::new(self.decode_section()?).decode()?)
        }
        SectionCode::Custom | SectionCode::Import | SectionCode::Start => {
          unimplemented!("{:?}", code);
        }
      };
    }
    Ok(section.complete()?)
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
        let mut bc = Byte::new_with_drop(buffer);
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
