use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::convert::From;
use core::default::Default;
use core::iter::Iterator;
use core::slice::Iter;
use decode::TableType;
use function::{FunctionInstance, FunctionType};
use global::{GlobalInstance, GlobalInstances, GlobalType};
use hashbrown::HashMap;
use inst::Indice;
use memory::{Limit, MemoryInstance, MemoryInstances};
use store::Store;
use table::{TableInstance, TableInstances};
use trap::{Result, Trap};

#[derive(Debug, Clone)]
pub enum ImportDescriptor {
  Function(u32), // NOTE: Index of FunctionTypes
  Table(TableType),
  Memory(Limit),
  Global(GlobalType),
}

#[derive(Debug, Clone)]
pub enum ExportDescriptor {
  Function(u32), // FIXME: Use Indice type.
  Table(u32),
  Memory(u32),
  Global(u32),
}

impl From<(Option<u8>, u32)> for ExportDescriptor {
  fn from(codes: (Option<u8>, u32)) -> Self {
    use self::ExportDescriptor::*;
    match codes.0 {
      Some(0x00) => Function(codes.1),
      Some(0x01) => Table(codes.1),
      Some(0x02) => Memory(codes.1),
      Some(0x03) => Global(codes.1),
      x => unreachable!("Expected exports descriptor, got {:?}", x),
    }
  }
}

#[derive(Debug, Clone)]
pub enum ModuleDescriptor {
  ImportDescriptor(ImportDescriptor),
  ExportDescriptor(ExportDescriptor),
}

// FIXME: As using simply label, prefer to define constant of those variants.
#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum ModuleDescriptorKind {
  Function,
  Table,
  Memory,
  Global,
}

impl From<Option<u8>> for ModuleDescriptorKind {
  fn from(code: Option<u8>) -> Self {
    use self::ModuleDescriptorKind::*;
    match code {
      Some(0x0) => Function,
      Some(0x1) => Table,
      Some(0x2) => Memory,
      Some(0x3) => Global,
      x => unreachable!("Expected import descriptor, got {:x?}", x),
    }
  }
}

pub type ModuleName = Option<String>;
type Name = String;

#[derive(Debug, Clone)]
pub struct ExternalInterface {
  pub module_name: ModuleName,
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
pub struct ExternalInterfaces(Vec<ExternalInterface>);

impl ExternalInterfaces {
  pub fn push(&mut self, value: ExternalInterface) {
    self.0.push(value);
  }

  pub fn find_kind_by_idx(
    &self,
    idx: u32,
    kind: &ModuleDescriptorKind,
  ) -> Option<&ExternalInterface> {
    self
      .0
      .iter()
      .find(|ExternalInterface { descriptor, .. }| match descriptor {
        ModuleDescriptor::ExportDescriptor(ExportDescriptor::Function(x)) => {
          &ModuleDescriptorKind::Function == kind && *x == idx
        }
        ModuleDescriptor::ExportDescriptor(ExportDescriptor::Table(x)) => {
          &ModuleDescriptorKind::Table == kind && *x == idx
        }
        ModuleDescriptor::ExportDescriptor(ExportDescriptor::Memory(x)) => {
          &ModuleDescriptorKind::Memory == kind && *x == idx
        }
        ModuleDescriptor::ExportDescriptor(ExportDescriptor::Global(x)) => {
          &ModuleDescriptorKind::Global == kind && *x == idx
        }
        _ => unreachable!(),
      })
  }

  pub fn iter(&self) -> Iter<ExternalInterface> {
    self.0.iter()
  }

  pub fn group_by_kind(&self) -> HashMap<ModuleDescriptorKind, Vec<ExternalInterface>> {
    let mut buf_function = vec![];
    let mut buf_table = vec![];
    let mut buf_memory = vec![];
    let mut buf_global = vec![];
    let mut buf = HashMap::new();

    for x in self.iter() {
      match &x.descriptor {
        ModuleDescriptor::ImportDescriptor(ImportDescriptor::Function(_)) => {
          buf_function.push(x.clone());
        }
        ModuleDescriptor::ImportDescriptor(ImportDescriptor::Table(_)) => {
          buf_table.push(x.clone());
        }
        ModuleDescriptor::ImportDescriptor(ImportDescriptor::Memory(_)) => {
          buf_memory.push(x.clone());
        }
        ModuleDescriptor::ImportDescriptor(ImportDescriptor::Global(_)) => {
          buf_global.push(x.clone());
        }
        y => unimplemented!("{:?}", y),
      };
    }
    buf.insert(ModuleDescriptorKind::Function, buf_function);
    buf.insert(ModuleDescriptorKind::Table, buf_table);
    buf.insert(ModuleDescriptorKind::Memory, buf_memory);
    buf.insert(ModuleDescriptorKind::Global, buf_global);
    buf
  }
}

impl Default for ExternalInterfaces {
  fn default() -> Self {
    ExternalInterfaces(vec![])
  }
}

#[derive(Debug)]
pub struct InternalModule {
  exports: ExternalInterfaces,
  pub start: Option<Indice>,
}

impl InternalModule {
  pub fn new(exports: ExternalInterfaces, start: Option<u32>) -> Self {
    InternalModule {
      exports,
      start: start.map(Indice::from),
    }
  }

  pub fn get_export_by_key(&self, invoke: &str) -> Option<&ExternalInterface> {
    self.exports.0.iter().find(|x| x.name == invoke)
  }
}

#[derive(Debug, Clone)]
pub struct ExternalModule {
  pub function_instances: Vec<FunctionInstance>,
  function_types: Vec<FunctionType>,
  pub(crate) memory_instances: MemoryInstances,
  table_instances: TableInstances,
  global_instances: GlobalInstances,
}

impl ExternalModule {
  // NOTE: Only for spectest
  pub(crate) fn new(
    function_instances: Vec<FunctionInstance>,
    function_types: Vec<FunctionType>,
    memory_instances: Vec<MemoryInstance>,
    table_instances: Vec<TableInstance>,
    global_instances: Vec<GlobalInstance>,
  ) -> Self {
    ExternalModule {
      function_instances,
      function_types,
      memory_instances: MemoryInstances::new(memory_instances),
      table_instances: TableInstances::new(table_instances),
      global_instances: GlobalInstances::new(global_instances),
    }
  }

  // FIXME: Consider to rename import-function-instance
  fn find_function_instance(
    &self,
    key: &ExternalInterface,
    function_types: &[FunctionType],
  ) -> Result<FunctionInstance> {
    match key {
      ExternalInterface {
        descriptor: ModuleDescriptor::ImportDescriptor(ImportDescriptor::Function(idx)),
        name,
        module_name,
      } => {
        let expected_type = function_types.get(*idx as usize)?;
        let instance = self
          .function_instances
          .iter()
          .find(|instance| instance.is_same_name(name))
          .ok_or(Trap::UnknownImport)
          .map(|x| x.clone())?;

        instance
          .validate_type(expected_type)
          .map_err(|_| Trap::IncompatibleImportType)?;

        instance.set_source_module_name(module_name);
        Ok(instance)
      }
      x => unreachable!("Expected function descriptor, got {:?}", x),
    }
  }

  fn find_table_instance(
    &self,
    key: &ExternalInterface, // import section of table
  ) -> Result<TableInstances> {
    match key {
      ExternalInterface {
        descriptor: ModuleDescriptor::ImportDescriptor(ImportDescriptor::Table(table_type)),
        name,
        ..
      } => {
        if !self.table_instances.find_by_name(name) {
          return Err(Trap::UnknownImport);
        }
        if self.table_instances.gt_table_type(table_type) {
          return Err(Trap::IncompatibleImportType);
        }
        Ok(self.table_instances.clone())
      }
      x => unreachable!("Expected table descriptor, got {:?}", x),
    }
  }
}

impl Default for ExternalModule {
  fn default() -> Self {
    ExternalModule {
      function_instances: vec![],
      function_types: vec![],
      memory_instances: MemoryInstances::empty(),
      table_instances: TableInstances::empty(),
      global_instances: GlobalInstances::empty(),
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
pub struct ExternalModules(Rc<RefCell<HashMap<ModuleName, ExternalModule>>>);

impl Default for ExternalModules {
  fn default() -> Self {
    ExternalModules(Rc::new(RefCell::new(HashMap::new())))
  }
}

impl ExternalModules {
  pub fn get(&self, module_name: &ModuleName) -> Option<ExternalModule> {
    self.0.borrow().get(module_name).cloned()
  }

  pub fn register_module(&mut self, key: ModuleName, value: ExternalModule) {
    self.0.borrow_mut().insert(key, value);
  }

  pub fn get_table_instance(&self, module_name: &ModuleName, idx: u32) -> Result<TableInstance> {
    self
      .0
      .borrow()
      .get(module_name)
      .ok_or(Trap::UnknownImport)?
      .table_instances
      .get_table_at(idx)
      .ok_or(Trap::Notfound)
  }

  pub fn get_function_type(&self, module_name: &ModuleName, idx: u32) -> Result<FunctionType> {
    self
      .0
      .borrow()
      .get(module_name)
      .ok_or(Trap::UnknownImport)?
      .function_types
      .get(idx as usize)
      .ok_or(Trap::Notfound)
      .map(|x| x.clone())
  }

  pub fn get_function_instance(
    &self,
    module_name: &ModuleName,
    idx: usize,
  ) -> Result<FunctionInstance> {
    self
      .0
      .borrow()
      .get(module_name)
      .ok_or(Trap::UnknownImport)?
      .function_instances
      .get(idx)
      .cloned()
      .ok_or(Trap::Notfound)
  }

  pub fn find_function_instances(
    &self,
    import: &ExternalInterface,
    function_types: &[FunctionType],
  ) -> Result<FunctionInstance> {
    self
      .0
      .borrow()
      .get(&import.module_name)
      .ok_or(Trap::UnknownImport)?
      .find_function_instance(import, function_types)
  }

  pub fn find_memory_instances(&self, import: &ExternalInterface) -> Result<MemoryInstances> {
    self
      .0
      .borrow()
      .get(&import.module_name)
      .ok_or(Trap::UnknownImport)
      .map(|x| x.memory_instances.clone())
  }

  pub fn find_table_instances(&self, import: &ExternalInterface) -> Result<TableInstances> {
    self
      .0
      .borrow()
      .get(&import.module_name)
      .ok_or(Trap::UnknownImport)?
      .find_table_instance(import)
  }

  pub fn find_global_instances(&self, module_name: &ModuleName) -> Result<GlobalInstances> {
    self
      .0
      .borrow()
      .get(module_name)
      .ok_or(Trap::UnknownImport)
      .map(|x| x.global_instances.clone())
  }
}
