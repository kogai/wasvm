use super::decodable::{Decodable, Leb128Decodable, U32Decodable};
use super::section::{Section, SectionCode};
use super::*;
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::default::Default;
use trap::{Result, Trap};

impl_decodable!(Byte);
impl Leb128Decodable for Byte {}
impl U32Decodable for Byte {}

impl Byte {
  pub fn new_with_drop(bytes: &[u8]) -> Result<Self> {
    if 4 > bytes.len() {
      return Err(Trap::UnexpectedEnd);
    }
    let (magic_words, bytes) = bytes.split_at(4);
    if magic_words.starts_with(&[40]) {
      return Err(Trap::UnsupportedTextform);
    }
    if magic_words != [0, 97, 115, 109] {
      return Err(Trap::MagicHeaderNotDetected);
    }
    if 4 > bytes.len() {
      return Err(Trap::UnexpectedEnd);
    }
    let (wasm_versions, bytes) = bytes.split_at(4);
    if wasm_versions != [1, 0, 0, 0] {
      return Err(Trap::UnsupportedTextform);
    }
    Ok(Byte::new(bytes.to_vec()))
  }

  fn has_next(&self) -> bool {
    self.byte_ptr < self.bytes.len()
  }

  // FIXME: It isn't guranteed whether bin_size_of_section actually can trusted or not.
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
  use embedder::{decode_module, init_store};
  use function::{FunctionInstance, FunctionType};
  use isa::Inst;
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
        let mut store = init_store();
        decode_module(&buffer)
          .unwrap()
          .complete(&ExternalModules::default(), &mut store)
          .unwrap();
        assert_eq!(
          store.get_function_instance(&From::from(0u32)).unwrap(),
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
      vec![
        I32Const,
        ExperimentalByte(42),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        End
      ],
    )
  );
  test_decode!(
    decode_cons16,
    "dist/cons16",
    FunctionInstance::new(
      Some("_subject".to_owned()),
      FunctionType::new(vec![], vec![ValueTypes::I32]),
      vec![],
      vec![
        I32Const,
        ExperimentalByte(255),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        End
      ],
    )
  );
  test_decode!(
    decode_signed,
    "dist/signed",
    FunctionInstance::new(
      Some("_subject".to_owned()),
      FunctionType::new(vec![], vec![ValueTypes::I32]),
      vec![],
      vec![
        I32Const,
        ExperimentalByte(127),
        ExperimentalByte(255),
        ExperimentalByte(255),
        ExperimentalByte(255),
        End
      ],
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
      vec![
        GetLocal,
        ExperimentalByte(1),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        GetLocal,
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I32Add,
        End
      ],
    )
  );
  test_decode!(
    decode_sub,
    "dist/sub",
    FunctionInstance::new(
      Some("_subject".to_owned()),
      FunctionType::new(vec![ValueTypes::I32], vec![ValueTypes::I32],),
      vec![],
      vec![
        I32Const,
        ExperimentalByte(100),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        GetLocal,
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I32Sub,
        End
      ],
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
      vec![
        GetLocal,
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I32Const,
        ExperimentalByte(10),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I32Add,
        GetLocal,
        ExperimentalByte(1),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I32Add,
        End
      ],
    )
  );

  test_decode!(
    decode_if_lt,
    "dist/if_lt",
    FunctionInstance::new(
      Some("_subject".to_owned()),
      FunctionType::new(vec![ValueTypes::I32], vec![ValueTypes::I32],),
      vec![ValueTypes::I32],
      vec![
        GetLocal,
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I32Const,
        ExperimentalByte(10),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        LessThanSign,
        If,
        ExperimentalByte(22),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(50),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0x7f),
        GetLocal,
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I32Const,
        ExperimentalByte(10),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I32Add,
        Else,
        GetLocal,
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I32Const,
        ExperimentalByte(15),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I32Add,
        SetLocal,
        ExperimentalByte(1),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        GetLocal,
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I32Const,
        ExperimentalByte(10),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        Equal,
        If,
        ExperimentalByte(16),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(6),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0x7f),
        I32Const,
        ExperimentalByte(15),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        Else,
        GetLocal,
        ExperimentalByte(1),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
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
      vec![
        GetLocal,
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I32Const,
        ExperimentalByte(10),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I32GreaterThanSign,
        If,
        ExperimentalByte(22),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(50),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0x7f),
        GetLocal,
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I32Const,
        ExperimentalByte(10),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I32Add,
        Else,
        GetLocal,
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I32Const,
        ExperimentalByte(15),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I32Add,
        SetLocal,
        ExperimentalByte(1),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        GetLocal,
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I32Const,
        ExperimentalByte(10),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        Equal,
        If,
        ExperimentalByte(16),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(6),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0x7f),
        I32Const,
        ExperimentalByte(15),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        Else,
        GetLocal,
        ExperimentalByte(1),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
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
      vec![
        GetLocal,
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I32Const,
        ExperimentalByte(10),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        Equal,
        If,
        ExperimentalByte(16),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(6),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0x7f),
        I32Const,
        ExperimentalByte(5),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        Else,
        I32Const,
        ExperimentalByte(10),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        End,
        GetLocal,
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
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
      vec![
        GetLocal,
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I32Const,
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I32LessEqualSign,
        If,
        ExperimentalByte(17),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0x40),
        I32Const,
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        Return,
        End,
        GetLocal,
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I32Const,
        ExperimentalByte(255),
        ExperimentalByte(255),
        ExperimentalByte(255),
        ExperimentalByte(255),
        I32Add,
        TeeLocal,
        ExperimentalByte(1),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        GetLocal,
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I32Const,
        ExperimentalByte(1),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I32Add,
        I32Mul,
        GetLocal,
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I32Add,
        GetLocal,
        ExperimentalByte(1),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I64ExtendUnsignI32,
        GetLocal,
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I32Const,
        ExperimentalByte(254),
        ExperimentalByte(255),
        ExperimentalByte(255),
        ExperimentalByte(255),
        I32Add,
        I64ExtendUnsignI32,
        I64Mul,
        I64Const,
        ExperimentalByte(255),
        ExperimentalByte(255),
        ExperimentalByte(255),
        ExperimentalByte(255),
        ExperimentalByte(1),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I64And,
        I64Const,
        ExperimentalByte(1),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        ExperimentalByte(0),
        I64ShiftRightUnsign,
        I32WrapI64,
        I32Add,
        End,
      ],
    )
  );
}
