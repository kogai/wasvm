use core::convert::From;
use core::option::NoneError;

pub mod runtime {
  #[cfg(not(test))]
  use alloc::prelude::*;
  use alloc::string::String;
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
}

pub mod validate_time {
  use super::runtime::Trap;

  #[cfg(not(test))]
  use alloc::prelude::*;
  use alloc::string::String;
  use core::convert::From;
  use core::option::NoneError;

  #[derive(Debug, PartialEq)]
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
        MultipleTables => "multiple tables".to_string(),
        MultipleMemories => "multiple memories".to_string(),
        TypeMismatch => "type mismatch".to_string(),
        IndirectCallTypeMismatch => "indirect call type mismatch".to_string(),
        IncompatibleImportType => "incompatible import type".to_string(),
        InvalidResultArity => "invalid result arity".to_string(),
        InvalidAlignment => "alignment must not be larger than natural".to_string(),
        InvalidMemorySize => "invalid memory size".to_string(),
        InvalidStartFunction => "invalid start function".to_string(),
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
}

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
      _ => unreachable!()
    }
  }
}

impl From<self::runtime::Trap> for WasmError {
  fn from(trap: self::runtime::Trap) -> Self {
    WasmError::Trap(trap)
  }
}

pub type Result<T> = core::result::Result<T, WasmError>;
