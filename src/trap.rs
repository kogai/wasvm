use std::convert::From;
use std::option::NoneError;

#[derive(Debug, Clone, PartialEq)]
pub enum Trap {
  DivisionOverflow,
  DivisionByZero,
  MemoryAccessOutOfBounds,
  BitshiftOverflow,
  Unknown,
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
      DivisionOverflow => "division overflow",
      DivisionByZero => "division by zero",
      MemoryAccessOutOfBounds => "memory access out of bounds",
      BitshiftOverflow => "bit shift overflow",
      Unknown => "unknown",
    }
    .to_owned()
  }
}

pub type Result<T> = std::result::Result<T, Trap>;
