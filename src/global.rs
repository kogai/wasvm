use value::Values;
use value_type::ValueTypes;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct GlobalInstance {
  global_type: GlobalType,
  value: Values,
  pub export_name: Option<String>,
}

impl GlobalInstance {
  pub fn new(global_type: GlobalType, value: Values, export_name: Option<String>) -> Self {
    GlobalInstance {
      global_type,
      value,
      export_name,
    }
  }
  pub fn get_value(&self) -> &Values {
    &self.value
  }
  pub fn set_value(&mut self, value: Values) {
    self.value = value;
  }
}
