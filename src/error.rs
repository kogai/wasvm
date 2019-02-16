use core::convert::From;
use core::option::NoneError;

pub mod runtime {
  use core::convert::From;
  use core::option::NoneError;

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
}

pub mod validate_time {
  use core::convert::From;
  use core::option::NoneError;

  #[derive(Debug, Clone, PartialEq)]
  pub enum TypeError {
    NotFound,
    MultipleTables,
    MultipleMemories,
    TypeMismatch,
    IndirectCallTypeMismatch,
    IncompatibleImportType,
    InvalidResultArity,
    InvalidAlignment,
    InvalidMemorySize,
    InvalidStartFunction,
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum WasmError {
  Trap(self::runtime::Trap),
  TypeError(self::validate_time::TypeError),
}

impl From<WasmError> for NoneError {
  fn from(_: WasmError) -> Self {
    NoneError
  }
}

impl From<NoneError> for WasmError {
  fn from(_: NoneError) -> Self {
    WasmError::Trap(self::runtime::Trap::Notfound)
  }
}

impl From<WasmError> for self::runtime::Trap {
  fn from(wasm_error: WasmError) -> Self {
    match wasm_error {
      WasmError::Trap(e) => e,
      _ => unreachable!(),
    }
  }
}

impl From<self::validate_time::TypeError> for WasmError {
  fn from(type_error: self::validate_time::TypeError) -> Self {
    WasmError::TypeError(type_error)
  }
}

impl From<WasmError> for self::validate_time::TypeError {
  fn from(wasm_error: WasmError) -> Self {
    match wasm_error {
      WasmError::TypeError(e) => e,
      _ => unreachable!(),
    }
  }
}

impl From<self::runtime::Trap> for WasmError {
  fn from(trap: self::runtime::Trap) -> Self {
    WasmError::Trap(trap)
  }
}

pub type Result<T> = core::result::Result<T, WasmError>;
