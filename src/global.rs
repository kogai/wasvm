use code::ValueTypes;
use inst::Instructions;

pub struct GlobalInstance {
  global_type: GlobalType,
  init: Instructions,
}

impl GlobalInstance {
  pub fn new(global_type: GlobalType, init: Instructions) -> Self {
    GlobalInstance { global_type, init }
  }
}

pub enum GlobalType {
  Const(ValueTypes),
  Var(ValueTypes),
}

impl GlobalType {
  pub fn new(code: Option<u8>, v: ValueTypes) -> Self {
    match code {
      Some(0x00) => GlobalType::Const(v),
      Some(0x01) => GlobalType::Var(v),
      x => unreachable!("Expected global type code, got {:?}", x),
    }
  }
}
