use decode::TableInstance;
use function::{FunctionInstance, FunctionType};
use global::GlobalInstance;
use memory::MemoryInstance;
use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::convert::From;
use std::default::Default;
use std::iter::Iterator;
use std::rc::Rc;
use store::Store;
use trap::{Result, Trap};

#[derive(Debug, Clone)]
pub enum ModuleDescriptor {
  Function(u32), // NOTE: Index of FunctionTypes
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

type ModuleName = Option<String>;
type Name = String;

#[derive(Debug, Clone)]
pub struct ExternalInterface {
  module_name: ModuleName,
  pub name: Name,
  pub descriptor: ModuleDescriptor,
}

impl ExternalInterface {
  pub fn new(module_name: ModuleName, name: Name, descriptor: ModuleDescriptor) -> Self {
    ExternalInterface {
      module_name,
      name,
      descriptor,
    }
  }
}

#[derive(Debug)]
pub struct ExternalInterfaces(HashMap<(ModuleName, Name), ExternalInterface>);

impl ExternalInterfaces {
  pub fn new() -> Self {
    ExternalInterfaces(HashMap::new())
  }

  pub fn insert(&mut self, value: ExternalInterface) {
    self
      .0
      .insert((value.module_name.clone(), value.name.clone()), value);
  }

  pub fn find_function_by_idx(&self, idx: u32) -> Option<&ExternalInterface> {
    self
      .0
      .iter()
      .find(
        |(_key, ExternalInterface { descriptor, .. })| match descriptor {
          ModuleDescriptor::Function(x) => *x == idx,
          _ => false,
        },
      )
      .map(|(_, x)| x)
  }

  pub fn iter(&self) -> Iter<(ModuleName, Name), ExternalInterface> {
    self.0.iter()
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
    self.exports.0.get(&(None, invoke.to_owned()))
  }
}

#[derive(Debug, Clone)]
pub struct ExternalModule {
  pub function_instances: Vec<Rc<FunctionInstance>>,
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

  pub fn find_function_instance(
    &self,
    key: &ExternalInterface,
    function_types: &Vec<FunctionType>,
  ) -> Result<Rc<FunctionInstance>> {
    match key {
      ExternalInterface {
        descriptor: ModuleDescriptor::Function(idx),
        name,
        ..
      } => {
        let instance = self
          .function_instances
          .iter()
          .find(|instance| instance.export_name == Some(name.to_owned()))
          .map(|x| x.clone())
          .ok_or(Trap::UnknownImport)?;
        let expected_type = function_types.get(*idx as usize)?;
        instance
          .validate_type(expected_type)
          .map_err(|_| Trap::IncompatibleImportType)?;
        Ok(instance)
      }
      x => unreachable!("Expected function descriptor, got {:?}", x),
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

#[derive(Debug, Clone)]
pub struct ExternalModules(HashMap<ModuleName, ExternalModule>);

impl ExternalModules {
  pub fn new() -> Self {
    ExternalModules(HashMap::new())
  }

  pub fn register_module(&mut self, key: ModuleName, value: ExternalModule) {
    self.0.insert(key, value);
  }

  pub fn get(&self, key: &ModuleName) -> Option<&ExternalModule> {
    self.0.get(key)
  }
}
