use alloc::prelude::*;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::fmt;
use inst::Inst;
use module::ModuleName;
use stack::StackEntry;
use trap::{Result, Trap};
use value::Values;
use value_type::ValueTypes;

#[derive(PartialEq, Clone)]
pub struct FunctionType {
  parameters: Vec<ValueTypes>,
  returns: Vec<ValueTypes>,
}

impl fmt::Debug for FunctionType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "({}) -> ({})",
      self
        .parameters
        .iter()
        .map(|p| format!("{:?}", p))
        .collect::<Vec<String>>()
        .join(", "),
      self
        .returns
        .iter()
        .map(|p| format!("{:?}", p))
        .collect::<Vec<String>>()
        .join(", "),
    )
  }
}

impl FunctionType {
  pub fn new(parameters: Vec<ValueTypes>, returns: Vec<ValueTypes>) -> Self {
    FunctionType {
      parameters,
      returns,
    }
  }

  pub fn get_parameter_types<'a>(&'a self) -> &'a Vec<ValueTypes> {
    &self.parameters
  }

  pub fn get_return_types<'a>(&'a self) -> &'a Vec<ValueTypes> {
    &self.returns
  }

  pub fn get_arity(&self) -> u32 {
    self.parameters.len() as u32
  }
}

#[derive(PartialEq)]
struct FunctionInstanceImpl {
  export_name: Option<String>,
  function_type: FunctionType,
  local_variables: Vec<Rc<StackEntry>>,
  body: Vec<Inst>,
  source_module_name: RefCell<Option<String>>,
}

// FIXME: Add enum which represents either FunctionInstance or HostFunction.
#[derive(Clone, PartialEq)]
pub struct FunctionInstance(Rc<FunctionInstanceImpl>);

impl FunctionInstance {
  pub fn new(
    export_name: Option<String>,
    function_type: FunctionType,
    mut locals: Vec<ValueTypes>,
    body: Vec<Inst>,
  ) -> Self {
    locals.reverse();
    let local_variables = locals
      .iter()
      .map(|local| StackEntry::new_value(Values::from(local)))
      .collect::<Vec<_>>();
    FunctionInstance(Rc::new(FunctionInstanceImpl {
      export_name,
      function_type,
      local_variables,
      body,
      source_module_name: RefCell::new(None),
    }))
  }

  pub fn local_variables(&self) -> Vec<Rc<StackEntry>> {
    self.0.local_variables.clone()
  }

  pub fn function_type_ref(&self) -> &FunctionType {
    &self.0.function_type
  }

  pub fn set_source_module_name(&self, name: &ModuleName) {
    if let Some(name) = name {
      let mut source_module_name = self.0.source_module_name.borrow_mut();
      source_module_name.replace(name.to_owned());
    };
  }

  pub fn get_source_module_name(&self) -> Option<String> {
    self.0.source_module_name.borrow().to_owned()
  }

  pub fn get(&self, idx: usize) -> Option<&Inst> {
    self.0.body.get(idx)
  }

  pub fn get_expressions_count(&self) -> usize {
    self.0.body.len()
  }

  pub fn get_arity(&self) -> u32 {
    self.0.function_type.parameters.len() as u32
  }

  pub fn get_function_type(&self) -> FunctionType {
    self.0.function_type.to_owned()
  }

  pub fn get_return_type(&self) -> &Vec<ValueTypes> {
    &self.0.function_type.returns
  }

  pub fn get_return_count(&self) -> u32 {
    self.0.function_type.returns.len() as u32
  }

  pub fn validate_type(&self, other: &FunctionType) -> Result<()> {
    if &self.0.function_type != other {
      Err(Trap::TypeMismatch)
    } else {
      Ok(())
    }
  }

  pub fn is_same_name(&self, other_name: &str) -> bool {
    self.0.export_name.as_ref() == Some(&other_name.to_string())
  }
}

impl fmt::Debug for FunctionInstance {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_struct("FunctionInstance")
      .field(
        "export_name",
        &match self.0.export_name {
          Some(ref n) => n,
          _ => "_",
        },
      )
      .field("function_type", &self.0.function_type)
      .field("instructions", &format_args!("{:?}", self.0.body))
      .finish()
  }
}
