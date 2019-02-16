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

#[derive(Debug, Clone, PartialEq)]
pub enum WasmError {
  Trap(Trap),
  TypeError(TypeError),
}

impl From<WasmError> for NoneError {
  fn from(_: WasmError) -> Self {
    NoneError
  }
}

impl From<NoneError> for WasmError {
  fn from(_: NoneError) -> Self {
    WasmError::Trap(Trap::Notfound)
  }
}

impl From<WasmError> for self::Trap {
  fn from(wasm_error: WasmError) -> Self {
    match wasm_error {
      WasmError::Trap(e) => e,
      _ => unreachable!(),
    }
  }
}

impl From<Trap> for WasmError {
  fn from(trap: Trap) -> Self {
    WasmError::Trap(trap)
  }
}

impl From<TypeError> for WasmError {
  fn from(type_error: TypeError) -> Self {
    WasmError::TypeError(type_error)
  }
}

impl From<WasmError> for TypeError {
  fn from(wasm_error: WasmError) -> Self {
    match wasm_error {
      WasmError::TypeError(e) => e,
      _ => unreachable!(),
    }
  }
}

pub type Result<T> = core::result::Result<T, WasmError>;
