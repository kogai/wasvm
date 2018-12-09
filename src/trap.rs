use std::convert::From;
use std::option::NoneError;

// TODO: Prefer to separate runtime error and decode-time one.
#[derive(Debug, Clone, PartialEq)]
pub enum Trap {
  DivisionOverflow,
  DivisionByZero,
  MemoryAccessOutOfBounds,
  BitshiftOverflow,
  Unknown,
  StackOverflow,
  StackUnderflow,
  Notfound,
  Undefined,
  TypeMismatch,
}

impl From<Trap> for NoneError {
  fn from(_: Trap) -> Self {
    NoneError
  }
}

impl From<NoneError> for Trap {
  fn from(_: NoneError) -> Self {
    Trap::Unknown
  }
}

impl From<Trap> for String {
  fn from(x: Trap) -> Self {
    use self::Trap::*;
    match x {
      DivisionOverflow => "integer overflow",
      DivisionByZero => "integer divide by zero",
      MemoryAccessOutOfBounds => "out of bounds memory access",
      BitshiftOverflow => "bit shift overflow",
      Unknown => "unknown",
      Undefined => "undefined behavior occurred",
      Notfound => "not found",
      StackOverflow => "stack overflow",
      StackUnderflow => "stack underflow",
      TypeMismatch => "type mismatch",
    }
    .to_owned()
  }
}

pub type Result<T> = std::result::Result<T, Trap>;
