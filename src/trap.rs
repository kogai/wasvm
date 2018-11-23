use std::convert::From;
use std::option::NoneError;

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
