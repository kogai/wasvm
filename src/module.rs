use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::convert::From;
use core::default::Default;
use core::fmt;
use core::iter::Iterator;
use core::slice::Iter;
use decode::TableType;
use error::{Result, Trap, WasmError};
use function::{FunctionInstance, FunctionType};
use global::{GlobalInstance, GlobalInstances, GlobalType};
use heapless::consts::{U32, U4};
use heapless::LinearMap;
use indice::Indice;
use memory::{Limit, MemoryInstance, MemoryInstances};
use store::Store;
use table::{TableInstance, TableInstances};

#[derive(Debug, Clone)]
pub enum ImportDescriptor {
  Function(Indice), // NOTE: Index of FunctionTypes
  Table(TableType),
  Memory(Limit),
  Global(GlobalType),
}

#[derive(Debug, Clone)]
pub enum ExportDescriptor {
  Function(Indice),
  Table(Indice),
  Memory(Indice),
  Global(Indice),
}

impl From<(Option<u8>, u32)> for ExportDescriptor {
  fn from(codes: (Option<u8>, u32)) -> Self {
    use self::ExportDescriptor::*;
    match codes.0 {
      Some(0x00) => Function(From::from(codes.1)),
      Some(0x01) => Table(From::from(codes.1)),
      Some(0x02) => Memory(From::from(codes.1)),
      Some(0x03) => Global(From::from(codes.1)),
      x => unreachable!("Expected exports descriptor, got {:?}", x),
    }
  }
}

#[derive(Debug, Clone)]
pub enum ModuleDescriptor {
  ImportDescriptor(ImportDescriptor),
  ExportDescriptor(ExportDescriptor),
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum ModuleDescriptorKind {
  Function,
  Table,
  Memory,
  Global,
}

pub const FUNCTION_DESCRIPTOR: ModuleDescriptorKind = ModuleDescriptorKind::Function;
pub const TABLE_DESCRIPTOR: ModuleDescriptorKind = ModuleDescriptorKind::Table;
pub const MEMORY_DESCRIPTOR: ModuleDescriptorKind = ModuleDescriptorKind::Memory;
pub const GLOBAL_DESCRIPTOR: ModuleDescriptorKind = ModuleDescriptorKind::Global;

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

  pub fn len(&self) -> usize {
    self.0.len()
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
          &FUNCTION_DESCRIPTOR == kind && x.to_u32() == idx
        }
        ModuleDescriptor::ExportDescriptor(ExportDescriptor::Table(x)) => {
          &TABLE_DESCRIPTOR == kind && x.to_u32() == idx
        }
        ModuleDescriptor::ExportDescriptor(ExportDescriptor::Memory(x)) => {
          &MEMORY_DESCRIPTOR == kind && x.to_u32() == idx
        }
        ModuleDescriptor::ExportDescriptor(ExportDescriptor::Global(x)) => {
          &GLOBAL_DESCRIPTOR == kind && x.to_u32() == idx
        }
        _ => unreachable!(),
      })
  }

  pub fn iter(&self) -> Iter<ExternalInterface> {
    self.0.iter()
  }

  pub fn group_by_kind(
    &self,
  ) -> Result<LinearMap<ModuleDescriptorKind, Vec<ExternalInterface>, U4>> {
    let mut buf_function = vec![];
    let mut buf_table = vec![];
    let mut buf_memory = vec![];
    let mut buf_global = vec![];
    let mut buf = LinearMap::new();

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
    buf
      .insert(ModuleDescriptorKind::Function, buf_function)
      .map_err(|_| Trap::LinearMapOverflowed)?;
    buf
      .insert(ModuleDescriptorKind::Table, buf_table)
      .map_err(|_| Trap::LinearMapOverflowed)?;
    buf
      .insert(ModuleDescriptorKind::Memory, buf_memory)
      .map_err(|_| Trap::LinearMapOverflowed)?;
    buf
      .insert(ModuleDescriptorKind::Global, buf_global)
      .map_err(|_| Trap::LinearMapOverflowed)?;
    Ok(buf)
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
  pub fn new(
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
        let expected_type = function_types.get(idx.to_usize())?;
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
          return Err(WasmError::Trap(Trap::UnknownImport));
        }
        if self.table_instances.gt_table_type(table_type) {
          return Err(WasmError::Trap(Trap::IncompatibleImportType));
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

#[derive(Clone)]
pub struct ExternalModules(Rc<RefCell<LinearMap<ModuleName, ExternalModule, U32>>>);

impl fmt::Debug for ExternalModules {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_map()
      .entries(self.0.borrow().iter().map(|(k, v)| (k, v)))
      .finish()
  }
}

impl Default for ExternalModules {
  fn default() -> Self {
    ExternalModules(Rc::new(RefCell::new(LinearMap::new())))
  }
}

impl ExternalModules {
  pub fn get(&self, module_name: &ModuleName) -> Option<ExternalModule> {
    self.0.borrow().get(module_name).cloned()
  }

  pub fn register_module(&mut self, key: ModuleName, value: ExternalModule) -> Result<()> {
    self
      .0
      .borrow_mut()
      .insert(key, value)
      .map_err(|_| Trap::LinearMapOverflowed)?;
    Ok(())
  }

  pub fn get_table_instance(
    &self,
    module_name: &ModuleName,
    idx: &Indice,
  ) -> Result<TableInstance> {
    self
      .0
      .borrow()
      .get(module_name)
      .ok_or(Trap::UnknownImport)?
      .table_instances
      .get_table_at(idx)
      .ok_or(WasmError::Trap(Trap::Notfound))
  }

  pub fn get_function_type(&self, module_name: &ModuleName, idx: u32) -> Result<FunctionType> {
    self
      .0
      .borrow()
      .get(module_name)
      .ok_or(WasmError::Trap(Trap::UnknownImport))?
      .function_types
      .get(idx as usize)
      .ok_or(WasmError::Trap(Trap::Notfound))
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
      .ok_or(WasmError::Trap(Trap::Notfound))
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
      .ok_or(WasmError::Trap(Trap::UnknownImport))
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
      .ok_or(WasmError::Trap(Trap::UnknownImport))
      .map(|x| x.global_instances.clone())
  }
}
