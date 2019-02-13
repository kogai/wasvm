#[cfg(not(test))]
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
  InvalidMutability,
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
  UninitializedElement,
  IncompatibleImportType,
  MagicHeaderNotDetected,
  UnsupportedTextform,
  IntegerRepresentationTooLong,
  FunctionAndCodeInconsitent,
  InvalidUTF8Encoding,
  LinearMapOverflowed,
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
      DivisionOverflow => "integer overflow",
      DivisionByZero => "integer divide by zero",
      DataSegmentDoesNotFit => "data segment does not fit",
      ElementSegmentDoesNotFit => "elements segment does not fit",
      MemoryAccessOutOfBounds => "out of bounds memory access",
      BitshiftOverflow => "bit shift overflow",
      IntegerOverflow => "integer overflow",
      Unknown => "unknown",
      Undefined => "undefined behavior occurred",
      UndefinedElement => "undefined element",
      Notfound => "not found",
      StackOverflow => "stack overflow",
      StackUnderflow => "stack underflow",
      TypeMismatch => "type mismatch",
      IndirectCallTypeMismatch => "indirect call type mismatch",
      InvalidMutability => "invalid mutability",
      FailToGrow => "fail to grow",
      InvalidConversionToInt => "invalid conversion to integer",
      UnexpectedEnd => "unexpected end",
      InvalidSectionId => "invalid section id",
      LengthOutofBounds => "length out of bounds",
      Unreachable => "unreachable executed",
      UnknownImport => "unknown import",
      IncompatibleImportType => "incompatible import type",
      UninitializedElement => "uninitialized element",
      UnsupportedTextform => "unsupported text form",
      MagicHeaderNotDetected => "magic header not detected",
      IntegerRepresentationTooLong => "integer representation too long",
      FunctionAndCodeInconsitent => "function and code section have inconsistent lengths",
      InvalidUTF8Encoding => "invalid UTF-8 encoding",
      LinearMapOverflowed => "LinearMap has been overflowed",
    }
    .to_owned()
  }
}

pub type Result<T> = core::result::Result<T, Trap>;
