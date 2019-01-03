use super::decodable::Decodable;
use super::section::{Section, SectionCode};
use super::*;
use core::convert::TryFrom;
use core::default::Default;
use trap::{Result, Trap};

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
    if end > self.bytes.len() {
      return Err(Trap::LengthOutofBounds);
    }
    let bytes = self.bytes.drain(start..end).collect::<Vec<_>>();
    Ok(bytes)
  }

  pub fn decode(&mut self) -> Result<Section> {
    use self::SectionCode::*;
    let mut section = Section::default();
    while self.has_next() {
      let code = SectionCode::try_from(self.next())?;
      let bytes = self.decode_section()?;
      // TODO: May can conccurrent.
      match code {
        Type => section.function_types(&mut sec_type::Section::new(bytes).decode()?),
        Function => section.functions(&mut sec_function::Section::new(bytes).decode()?),
        Code => section.codes(&mut sec_code::Section::new(bytes).decode()?),
        Data => section.datas(&mut sec_data::Section::new(bytes).decode()?),
        Memory => section.limits(&mut sec_memory::Section::new(bytes).decode()?),
        Table => section.tables(&mut sec_table::Section::new(bytes).decode()?),
        Global => section.globals(&mut sec_global::Section::new(bytes).decode()?),
        Element => section.elements(&mut sec_element::Section::new(bytes).decode()?),
        Custom => section.customs(&mut sec_custom::Section::new(bytes).decode()?),
        Export => section.exports(sec_export::Section::new(bytes).decode()?),
        Import => section.imports(sec_import::Section::new(bytes).decode()?),
        Start => section.start(sec_start::Section::new(bytes).decode()?),
      };
    }
    Ok(section)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use function::{FunctionInstance, FunctionType};
  use inst::Inst;
  use module::ExternalModules;
  use std::fs::File;
  use std::io::Read;
  use value_type::ValueTypes;

  macro_rules! test_decode {
    ($fn_name:ident, $file_name:expr, $fn_insts: expr) => {
      #[test]
      fn $fn_name() {
        use self::Inst::*;
        let mut file = File::open(format!("./{}.wasm", $file_name)).unwrap();
        let mut buffer = vec![];
        let _ = file.read_to_end(&mut buffer);
        let mut bc = Byte::new_with_drop(buffer);
        assert_eq!(
          bc.decode()
            .unwrap()
            .complete(ExternalModules::new())
            .unwrap()
            .0
            .get_function_instance(0)
            .unwrap(),
          $fn_insts
        );
      }
    };
  }

  test_decode!(
    decode_cons8,
    "dist/cons8",
    FunctionInstance::new(
      Some("_subject".to_owned()),
      FunctionType::new(vec![], vec![ValueTypes::I32]),
      vec![],
      0,
      vec![I32Const(42), End],
    )
  );
  test_decode!(
    decode_cons16,
    "dist/cons16",
    FunctionInstance::new(
      Some("_subject".to_owned()),
      FunctionType::new(vec![], vec![ValueTypes::I32]),
      vec![],
      0,
      vec![I32Const(255), End],
    )
  );
  test_decode!(
    decode_signed,
    "dist/signed",
    FunctionInstance::new(
      Some("_subject".to_owned()),
      FunctionType::new(vec![], vec![ValueTypes::I32]),
      vec![],
      0,
      vec![I32Const(-129), End],
    )
  );
  test_decode!(
    decode_add,
    "dist/add",
    FunctionInstance::new(
      Some("_subject".to_owned()),
      FunctionType::new(
        vec![ValueTypes::I32, ValueTypes::I32],
        vec![ValueTypes::I32],
      ),
      vec![],
      0,
      vec![GetLocal(1), GetLocal(0), I32Add, End],
    )
  );
  test_decode!(
    decode_sub,
    "dist/sub",
    FunctionInstance::new(
      Some("_subject".to_owned()),
      FunctionType::new(vec![ValueTypes::I32], vec![ValueTypes::I32],),
      vec![],
      0,
      vec![I32Const(100), GetLocal(0), I32Sub, End],
    )
  );
  test_decode!(
    decode_add_five,
    "dist/add_five",
    FunctionInstance::new(
      Some("_subject".to_owned()),
      FunctionType::new(
        vec![ValueTypes::I32, ValueTypes::I32],
        vec![ValueTypes::I32],
      ),
      vec![],
      0,
      vec![GetLocal(0), I32Const(10), I32Add, GetLocal(1), I32Add, End],
    )
  );

  test_decode!(
    decode_if_lt,
    "dist/if_lt",
    FunctionInstance::new(
      Some("_subject".to_owned()),
      FunctionType::new(vec![ValueTypes::I32], vec![ValueTypes::I32],),
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
    )
  );
  test_decode!(
    decode_if_gt,
    "dist/if_gt",
    FunctionInstance::new(
      Some("_subject".to_owned()),
      FunctionType::new(vec![ValueTypes::I32], vec![ValueTypes::I32],),
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
    )
  );
  test_decode!(
    decode_if_eq,
    "dist/if_eq",
    FunctionInstance::new(
      Some("_subject".to_owned()),
      FunctionType::new(vec![ValueTypes::I32], vec![ValueTypes::I32],),
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
    )
  );
  test_decode!(
    decode_count,
    "dist/count",
    FunctionInstance::new(
      Some("_subject".to_owned()),
      FunctionType::new(vec![ValueTypes::I32], vec![ValueTypes::I32],),
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
    )
  );
}
