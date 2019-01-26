use core::convert::From;
use core::fmt;

#[derive(PartialEq, Clone)]
pub enum ValueTypes {
  Empty, // TODO: Rename to Unit
  I32,
  I64,
  F32,
  F64,
}

pub const TYPE_I32: ValueTypes = ValueTypes::I32;
pub const TYPE_I64: ValueTypes = ValueTypes::I64;
pub const TYPE_F32: ValueTypes = ValueTypes::F32;
pub const TYPE_F64: ValueTypes = ValueTypes::F64;

// FIXME: Change implementation to From<&u8>
impl From<Option<u8>> for ValueTypes {
  fn from(code: Option<u8>) -> Self {
    match code {
      Some(0x40) => ValueTypes::Empty,
      Some(0x7f) => ValueTypes::I32,
      Some(0x7e) => ValueTypes::I64,
      Some(0x7d) => ValueTypes::F32,
      Some(0x7c) => ValueTypes::F64,
      x => unreachable!("Expected value type, got {:?}", x),
    }
  }
}

impl fmt::Debug for ValueTypes {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use self::ValueTypes::*;
    write!(
      f,
      "{}",
      match self {
        Empty => "*",
        I32 => "i32",
        I64 => "i64",
        F32 => "f32",
        F64 => "f64",
      }
    )
  }
}
