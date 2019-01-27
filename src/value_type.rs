use core::convert::From;
use core::fmt;

#[derive(PartialEq, Clone)]
pub enum ValueTypes {
  Unit,
  I32,
  I64,
  F32,
  F64,
}

pub const TYPE_UNIT: ValueTypes = ValueTypes::Unit;
pub const TYPE_I32: ValueTypes = ValueTypes::I32;
pub const TYPE_I64: ValueTypes = ValueTypes::I64;
pub const TYPE_F32: ValueTypes = ValueTypes::F32;
pub const TYPE_F64: ValueTypes = ValueTypes::F64;

impl From<u8> for ValueTypes {
  fn from(code: u8) -> Self {
    match code {
      0x40 => ValueTypes::Unit,
      0x7f => ValueTypes::I32,
      0x7e => ValueTypes::I64,
      0x7d => ValueTypes::F32,
      0x7c => ValueTypes::F64,
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
        Unit => "()",
        I32 => "i32",
        I64 => "i64",
        F32 => "f32",
        F64 => "f64",
      }
    )
  }
}
