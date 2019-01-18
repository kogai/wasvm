use alloc::string::String;
use core::convert::From;
use core::option::NoneError;
use trap::Trap;

#[derive(Debug, PartialEq)]
pub enum TypeError {
  NotFound,
  TypeMismatch,
  IndirectCallTypeMismatch,
  IncompatibleImportType,
  // FIXME: Separate TypeError and RuntimeError(Trap) completely.
  Trap(Trap),
}

impl From<TypeError> for NoneError {
  fn from(_: TypeError) -> Self {
    NoneError
  }
}

impl From<NoneError> for TypeError {
  fn from(_: NoneError) -> Self {
    TypeError::NotFound
  }
}

impl From<TypeError> for String {
  fn from(x: TypeError) -> Self {
    use self::TypeError::*;
    match x {
      NotFound => "not found".to_string(),
      TypeMismatch => "type mismatch".to_string(),
      IndirectCallTypeMismatch => "indirect call type mismatch".to_string(),
      IncompatibleImportType => "incompatible import type".to_string(),
      Trap(err) => String::from(err),
    }
  }
}

pub type Result<T> = core::result::Result<T, TypeError>;
