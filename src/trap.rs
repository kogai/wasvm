use alloc::prelude::*;
use alloc::string::String;
use core::convert::From;
use core::option::NoneError;

// TODO: Prefer to separate runtime error and decode-time one.
#[derive(Debug, Clone, PartialEq)]
pub enum Trap {
  DivisionOverflow,
  DivisionByZero,
  DataSegmentDoesNotFit,
  ElementSegmentDoesNotFit,
  MemoryAccessOutOfBounds,
  BitshiftOverflow,
  IntegerOverflow,
  InvalidConversionToInt,
  Unknown,
  StackOverflow,
  StackUnderflow,
  Notfound,
  Undefined,
  UndefinedElement,
  TypeMismatch,
  IndirectCallTypeMismatch,
  FailToGrow,
  UnexpectedEnd,
  InvalidSectionId,
  LengthOutofBounds,
  Unreachable,
  UnknownImport,
  UninitializedElement(u32),
  IncompatibleImportType,
  MagicHeaderNotDetected,
  UnsupportedTextform,
  IntegerRepresentationTooLong
}

impl From<Trap> for NoneError {
  fn from(_: Trap) -> Self {
    NoneError
  }
}

impl From<NoneError> for Trap {
  fn from(_: NoneError) -> Self {
    Trap::UnexpectedEnd
  }
}

impl From<Trap> for String {
  fn from(x: Trap) -> Self {
    use self::Trap::*;
    match x {
      DivisionOverflow => "integer overflow".to_owned(),
      DivisionByZero => "integer divide by zero".to_owned(),
      DataSegmentDoesNotFit => "data segment does not fit".to_owned(),
      ElementSegmentDoesNotFit => "elements segment does not fit".to_owned(),
      MemoryAccessOutOfBounds => "out of bounds memory access".to_owned(),
      BitshiftOverflow => "bit shift overflow".to_owned(),
      IntegerOverflow => "integer overflow".to_owned(),
      Unknown => "unknown".to_owned(),
      Undefined => "undefined behavior occurred".to_owned(),
      UndefinedElement => "undefined element".to_owned(),
      Notfound => "not found".to_owned(),
      StackOverflow => "stack overflow".to_owned(),
      StackUnderflow => "stack underflow".to_owned(),
      TypeMismatch => "type mismatch".to_owned(),
      IndirectCallTypeMismatch => "indirect call type mismatch".to_owned(),
      FailToGrow => "fail to grow".to_owned(),
      InvalidConversionToInt => "invalid conversion to integer".to_owned(),
      UnexpectedEnd => "unexpected end".to_owned(),
      InvalidSectionId => "invalid section id".to_owned(),
      LengthOutofBounds => "length out of bounds".to_owned(),
      Unreachable => "unreachable executed".to_owned(),
      UnknownImport => "unknown import".to_owned(),
      IncompatibleImportType => "incompatible import type".to_owned(),
      UninitializedElement(idx) => format!("uninitialized element {}", idx),
      UnsupportedTextform => "unsupported text form".to_owned(),
      MagicHeaderNotDetected => "magic header not detected".to_owned(),
      IntegerRepresentationTooLong => "integer representation too long".to_owned()
    }
  }
}

pub type Result<T> = core::result::Result<T, Trap>;
