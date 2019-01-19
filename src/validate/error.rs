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
  InvalidResultArity,
  InvalidAlignment,
  UnknownLabel,
  UnknownLocal,
  UnknownMemory,
  UnknownFunctionType(u32),
  UnknownFunction(u32),
  UnknownTable(u32),
  UnknownGlobal(u32),
  ConstantExpressionRequired,
  DuplicateExportName,
  GlobalIsImmutable,
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
      InvalidResultArity => "invalid result arity".to_string(),
      InvalidAlignment => "alignment must not be larger than natural".to_string(),
      UnknownLabel => "unknown label".to_string(),
      UnknownLocal => "unknown local".to_string(),
      UnknownMemory => "unknown memory 0".to_string(),
      ConstantExpressionRequired => "constant expression required".to_string(),
      UnknownFunction(idx) => format!("unknown function {}", idx),
      UnknownFunctionType(idx) => format!("unknown function type {}", idx),
      UnknownTable(idx) => format!("unknown table {}", idx),
      UnknownGlobal(idx) => format!("unknown global {}", idx),
      DuplicateExportName => "duplicate export name".to_string(),
      GlobalIsImmutable => "global is immutable".to_string(),
      Trap(err) => String::from(err),
    }
  }
}

pub type Result<T> = core::result::Result<T, TypeError>;
