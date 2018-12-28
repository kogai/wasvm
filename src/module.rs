use decode::TableInstance;
use function::{FunctionInstance, FunctionType};
use global::GlobalInstance;
use memory::MemoryInstance;
use std::collections::HashMap;
use std::convert::From;
use std::default::Default;
use std::rc::Rc;
use store::Store;

#[derive(Debug, Clone)]
pub enum ModuleDescriptor {
  Function(u32),
  Table(u32),
  Memory(u32),
  Global(u32),
}

impl From<(Option<u8>, u32)> for ModuleDescriptor {
  fn from(codes: (Option<u8>, u32)) -> Self {
    use self::ModuleDescriptor::*;
    match codes.0 {
      Some(0x0) => Function(codes.1),
      Some(0x1) => Table(codes.1),
      Some(0x2) => Memory(codes.1),
      Some(0x3) => Global(codes.1),
      x => unreachable!("Expected import descriptor, got {:?}", x),
    }
  }
}

#[derive(Debug, Clone)]
pub struct ExternalInterface {
  module_name: Option<String>,
  pub name: String,
  pub descriptor: ModuleDescriptor,
}

impl ExternalInterface {
  pub fn new(module_name: Option<String>, name: String, descriptor: ModuleDescriptor) -> Self {
    ExternalInterface {
      module_name,
      name,
      descriptor,
    }
  }
}

#[derive(Debug)]
pub struct ExternalInterfaces(HashMap<String, ExternalInterface>);

impl ExternalInterfaces {
  pub fn new() -> Self {
    ExternalInterfaces(HashMap::new())
  }

  pub fn insert(&mut self, key: String, value: ExternalInterface) {
    self.0.insert(key, value);
  }

  pub fn find_by_idx(&self, idx: u32) -> Option<&ExternalInterface> {
    self
      .0
      .iter()
      .find(
        |(_key, ExternalInterface { descriptor, .. })| match descriptor {
          ModuleDescriptor::Function(x)
          | ModuleDescriptor::Table(x)
          | ModuleDescriptor::Memory(x)
          | ModuleDescriptor::Global(x) => *x == idx,
        },
      )
      .map(|(_, x)| x)
  }
}

pub struct InternalModule {
  exports: ExternalInterfaces,
  imports: ExternalInterfaces,
}

impl InternalModule {
  pub fn new(exports: ExternalInterfaces, imports: ExternalInterfaces) -> Self {
    InternalModule { exports, imports }
  }

  pub fn get_export_by_key(&self, invoke: &str) -> Option<&ExternalInterface> {
    self.exports.0.get(invoke)
  }
}

#[derive(Clone)]
pub struct ExternalModule {
  function_instances: Vec<Rc<FunctionInstance>>,
  function_types: Vec<FunctionType>,
  memory_instances: Vec<MemoryInstance>,
  table_instances: Vec<TableInstance>,
  global_instances: Vec<GlobalInstance>,
}

impl ExternalModule {
  pub fn new(
    function_instances: Vec<Rc<FunctionInstance>>,
    function_types: Vec<FunctionType>,
    memory_instances: Vec<MemoryInstance>,
    table_instances: Vec<TableInstance>,
    global_instances: Vec<GlobalInstance>,
  ) -> Self {
    ExternalModule {
      function_instances,
      function_types,
      memory_instances,
      table_instances,
      global_instances,
    }
  }
}

impl Default for ExternalModule {
  fn default() -> Self {
    ExternalModule {
      function_instances: vec![],
      function_types: vec![],
      memory_instances: vec![],
      table_instances: vec![],
      global_instances: vec![],
    }
  }
}

impl From<&Store> for ExternalModule {
  fn from(store: &Store) -> Self {
    ExternalModule {
      function_instances: store.function_instances.clone(),
      function_types: store.function_types.clone(),
      memory_instances: store.memory_instances.clone(),
      table_instances: store.table_instances.clone(),
      global_instances: store.global_instances.clone(),
    }
  }
}

#[derive(Clone)]
pub struct ExternalModules(HashMap<String, ExternalModule>);

impl ExternalModules {
  pub fn new() -> Self {
    ExternalModules(HashMap::new())
  }

  pub fn register_module(&mut self, key: String, value: ExternalModule) {
    self.0.insert(key, value);
  }
}
