use code::ValueTypes;
use value::Values;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct GlobalInstance {
  global_type: GlobalType,
  value: Values,
}

impl GlobalInstance {
  pub fn new(global_type: GlobalType, value: Values) -> Self {
    GlobalInstance { global_type, value }
  }
  pub fn set_value(&mut self, value: Values) {
    self.value = value;
  }
}
