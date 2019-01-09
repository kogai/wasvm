use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use module::ModuleName;
use trap::Result;
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
  pub value: Values,
  pub export_name: Option<String>,
  source_module_name: RefCell<Option<String>>,
}

impl GlobalInstance {
  pub fn new(global_type: GlobalType, value: Values, export_name: Option<String>) -> Self {
    GlobalInstance {
      global_type,
      value,
      export_name,
      source_module_name: RefCell::new(None),
    }
  }
  pub fn get_value(&self) -> &Values {
    &self.value
  }
  pub fn set_value(&mut self, value: Values) {
    self.value = value;
  }

  pub fn set_source_module_name(&self, name: &ModuleName) {
    if let Some(name) = name {
      let mut source_module_name = self.source_module_name.borrow_mut();
      source_module_name.replace(name.to_owned());
    };
  }

  pub fn get_source_module_name(&self) -> Option<String> {
    self.source_module_name.borrow().to_owned()
  }
}
