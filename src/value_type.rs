use std::convert::From;
use std::fmt;

#[derive(PartialEq, Clone)]
pub enum ValueTypes {
  Empty, // TODO: Rename to Unit
  I32,
  I64,
  F32,
  F64,
}

impl From<Option<u8>> for ValueTypes {
  fn from(code: Option<u8>) -> Self {
    match code {
      Some(0x40) => ValueTypes::Empty,
      // Some(0x60) => TypeFunction,
      Some(0x7f) => ValueTypes::I32,
      Some(0x7e) => ValueTypes::I64,
      Some(0x7d) => ValueTypes::F32,
      Some(0x7c) => ValueTypes::F64,
      Some(x) => unimplemented!("ValueTypes of {:x} does not implemented yet.", x),
      None => unreachable!("ValueTypes not found"),
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
