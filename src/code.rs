use std::convert::From;

#[derive(Debug, PartialEq, Clone)]
pub enum Code {
  ConstI32,
  ConstI64,

  TypeValueI32,
  TypeValueEmpty,
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

  Call,
  Select,

  I32EqualZero,
  // TODO: Add prefix to indicate data-type like I32
  Equal,
  NotEqual,
  LessThanSign,
  LessThanUnsign,
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

impl From<Option<u8>> for Code {
  fn from(code: Option<u8>) -> Self {
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
      Some(0x40) => TypeValueEmpty,
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
      Some(0x7f) => TypeValueI32,
      Some(0x83) => I64And,
      Some(0x88) => I64ShiftRightUnsign,
      Some(0xa7) => I32WrapI64,
      Some(0xad) => I64ExtendUnsignI32,
      x => unreachable!("Code {:x?} does not supported yet.", x),
    }
  }
}

impl Code {
  pub fn is_end_of_code(code: Option<u8>) -> bool {
    match code {
      Some(0x0b) => true,
      _ => false,
    }
  }
}

#[derive(Debug, PartialEq, Clone)]
pub enum SecionCode {
  SectionType,
  SectionFunction,
  SectionExport,
  SectionCode,
}

impl From<Option<u8>> for SecionCode {
  fn from(code: Option<u8>) -> Self {
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

pub enum ExportDescriptionCode {
  ExportDescFunctionIdx,
  ExportDescTableIdx,
  ExportDescMemIdx,
  ExportDescGlobalIdx,
}

impl From<Option<u8>> for ExportDescriptionCode {
  fn from(code: Option<u8>) -> Self {
    use self::ExportDescriptionCode::*;
    match code {
      Some(0x00) => ExportDescFunctionIdx,
      Some(0x01) => ExportDescTableIdx,
      Some(0x02) => ExportDescMemIdx,
      Some(0x03) => ExportDescGlobalIdx,
      x => unreachable!("Export description code {:x?} does not supported yet.", x),
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

impl From<Option<u8>> for ValueTypes {
  fn from(code: Option<u8>) -> Self {
    match Code::from(code) {
      Code::TypeValueEmpty => ValueTypes::Empty,
      Code::TypeValueI32 => ValueTypes::I32,
      x => unimplemented!("ValueTypes of {:?} does not implemented yet.", x),
    }
  }
}
