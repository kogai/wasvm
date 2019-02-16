#[cfg(not(test))]
use alloc::prelude::*;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::fmt;
use error::{Result, TypeError, WasmError};
use module::ModuleName;
use stack::StackEntry;
use value::Values;
use value_type::ValueTypes;

#[derive(PartialEq, Clone)]
struct FunctionTypeImpl {
  parameters: Vec<ValueTypes>,
  returns: Vec<ValueTypes>,
}

#[derive(PartialEq, Clone)]
pub struct FunctionType(Rc<FunctionTypeImpl>);

impl FunctionType {
  pub fn new(parameters: Vec<ValueTypes>, returns: Vec<ValueTypes>) -> Self {
    FunctionType(Rc::new(FunctionTypeImpl {
      parameters,
      returns,
    }))
  }

  pub fn parameters<'a>(&'a self) -> &'a Vec<ValueTypes> {
    &self.0.parameters
  }

  pub fn returns<'a>(&'a self) -> &'a Vec<ValueTypes> {
    &self.0.returns
  }

  pub fn get_arity(&self) -> u32 {
    self.0.parameters.len() as u32
  }
}

impl fmt::Debug for FunctionType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "({}) -> ({})",
      self
        .0
        .parameters
        .iter()
        .map(|p| format!("{:?}", p))
        .collect::<Vec<String>>()
        .join(", "),
      self
        .0
        .returns
        .iter()
        .map(|p| format!("{:?}", p))
        .collect::<Vec<String>>()
        .join(", "),
    )
  }
}

#[derive(PartialEq)]
pub struct FunctionInstanceImpl {
  export_name: Option<String>,
  function_type: FunctionType,
  local_variables: Vec<StackEntry>,
  body: Vec<u8>,
  source_module_name: RefCell<Option<String>>,
}

impl FunctionInstanceImpl {
  pub fn get_expressions_count(&self) -> usize {
    self.body.len()
  }

  pub fn local_variables(&self) -> Vec<StackEntry> {
    self.local_variables.clone()
  }

  pub fn get(&self, idx: usize) -> Option<&u8> {
    self.body.get(idx)
  }

  pub(crate) fn body(&self) -> &[u8] {
    &self.body
  }
}

pub struct HostFunction {
  export_name: Option<String>,
  function_type: FunctionType,
  source_module_name: RefCell<Option<String>>,
  callable: &'static Fn(&[Values]) -> Vec<Values>,
}

impl HostFunction {
  pub(crate) fn call(&self, arguments: &[Values]) -> Vec<Values> {
    let callable = self.callable;
    callable(arguments)
  }
}

impl PartialEq for HostFunction {
  fn eq(&self, other: &HostFunction) -> bool {
    self.export_name == other.export_name
      && self.function_type == other.function_type
      && self.source_module_name == other.source_module_name
  }
}

#[derive(Clone, PartialEq)]
pub enum FunctionInstance {
  LocalFn(Rc<FunctionInstanceImpl>),
  HostFn(Rc<HostFunction>),
}

impl FunctionInstance {
  pub fn new(
    export_name: Option<String>,
    function_type: FunctionType,
    mut locals: Vec<ValueTypes>,
    body: Vec<u8>,
  ) -> Self {
    locals.reverse();
    let local_variables = locals
      .iter()
      .map(|local| StackEntry::new_value(Values::from(local)))
      .collect::<Vec<_>>();
    FunctionInstance::LocalFn(Rc::new(FunctionInstanceImpl {
      export_name,
      function_type,
      local_variables,
      body,
      source_module_name: RefCell::new(None),
    }))
  }

  pub fn new_host_fn<F>(
    export_name: Option<String>,
    function_type: FunctionType,
    callable: &'static F,
  ) -> Self
  where
    F: Fn(&[Values]) -> Vec<Values>,
  {
    FunctionInstance::HostFn(Rc::new(HostFunction {
      export_name,
      function_type,
      source_module_name: RefCell::new(None),
      callable,
    }))
  }

  pub fn function_type_ref(&self) -> &FunctionType {
    match self {
      FunctionInstance::LocalFn(f) => &f.function_type,
      FunctionInstance::HostFn(f) => &f.function_type,
    }
  }

  pub fn set_source_module_name(&self, name: &ModuleName) {
    if let Some(name) = name {
      let mut source_module_name = match self {
        FunctionInstance::LocalFn(f) => f.source_module_name.borrow_mut(),
        FunctionInstance::HostFn(f) => f.source_module_name.borrow_mut(),
      };
      source_module_name.replace(name.to_owned());
    };
  }

  pub fn get_source_module_name(&self) -> Option<String> {
    match self {
      FunctionInstance::LocalFn(f) => f.source_module_name.borrow().to_owned(),
      FunctionInstance::HostFn(f) => f.source_module_name.borrow().to_owned(),
    }
  }

  pub fn get_arity(&self) -> u32 {
    match self {
      FunctionInstance::LocalFn(f) => f.function_type.parameters().len() as u32,
      FunctionInstance::HostFn(f) => f.function_type.parameters().len() as u32,
    }
  }

  pub fn get_function_type(&self) -> FunctionType {
    match self {
      FunctionInstance::LocalFn(f) => f.function_type.to_owned(),
      FunctionInstance::HostFn(f) => f.function_type.to_owned(),
    }
  }

  pub fn get_return_type(&self) -> &Vec<ValueTypes> {
    match self {
      FunctionInstance::LocalFn(f) => f.function_type.returns(),
      FunctionInstance::HostFn(f) => f.function_type.returns(),
    }
  }

  pub fn get_return_count(&self) -> u32 {
    self.get_return_type().len() as u32
  }

  pub fn validate_type(&self, other: &FunctionType) -> Result<()> {
    let my = match self {
      FunctionInstance::LocalFn(f) => &f.function_type,
      FunctionInstance::HostFn(f) => &f.function_type,
    };
    if my != other {
      Err(WasmError::TypeError(TypeError::TypeMismatch))
    } else {
      Ok(())
    }
  }

  pub fn is_same_name(&self, other_name: &str) -> bool {
    let export_name = match self {
      FunctionInstance::LocalFn(f) => &f.export_name,
      FunctionInstance::HostFn(f) => &f.export_name,
    };
    export_name.as_ref() == Some(&other_name.to_string())
  }
}

impl fmt::Debug for FunctionInstance {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let empty: Vec<u8> = vec![];
    f.debug_struct("FunctionInstance")
      .field(
        "export_name",
        &match self {
          FunctionInstance::LocalFn(ref f) => match **f {
            FunctionInstanceImpl {
              export_name: Some(ref n),
              ..
            } => n,
            _ => "_",
          },
          FunctionInstance::HostFn(ref f) => match **f {
            HostFunction {
              export_name: Some(ref n),
              ..
            } => n,
            _ => "_",
          },
        },
      )
      .field(
        "function_type",
        &match self {
          FunctionInstance::LocalFn(f) => &f.function_type,
          FunctionInstance::HostFn(f) => &f.function_type,
        },
      )
      .field(
        "instructions",
        match self {
          FunctionInstance::LocalFn(f) => &f.body,
          FunctionInstance::HostFn(_) => &empty,
        },
      )
      .finish()
  }
}
