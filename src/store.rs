use alloc::rc::Rc;
use alloc::vec::Vec;
use core::default::Default;
use function::{FunctionInstance, FunctionType};
use global::GlobalInstances;
use memory::MemoryInstances;
use table::{TableInstance, TableInstances};
use trap::Result;
use value::Values;

#[derive(Debug)]
pub struct Store {
  pub function_instances: Vec<Rc<FunctionInstance>>,
  pub function_types: Vec<FunctionType>,
  pub memory_instances: MemoryInstances,
  pub table_instances: TableInstances,
  pub global_instances: GlobalInstances,
}

impl Store {
  pub fn new(
    function_instances: Vec<Rc<FunctionInstance>>,
    function_types: Vec<FunctionType>,
    memory_instances: MemoryInstances,
    table_instances: TableInstances,
    global_instances: GlobalInstances,
  ) -> Self {
    Store {
      function_instances,
      function_types,
      memory_instances,
      table_instances,
      global_instances,
    }
  }

  pub fn get_function_instance(&self, fn_idx: usize) -> Option<Rc<FunctionInstance>> {
    self.function_instances.get(fn_idx).cloned()
  }

  pub fn get_function_type(&self, idx: u32) -> Option<&FunctionType> {
    self.function_types.get(idx as usize)
  }

  pub fn get_function_type_by_instance(&self, idx: u32) -> Option<FunctionType> {
    self
      .get_function_instance(idx as usize)
      .map(|x| x.get_function_type())
  }

  pub fn get_global(&mut self, idx: u32) -> Result<Values> {
    self.global_instances.get_global(idx)
  }

  pub fn set_global(&mut self, idx: u32, value: Values) {
    self.global_instances.set_global(idx, value)
  }

  pub fn get_table_at(&self, idx: u32) -> Option<TableInstance> {
    self.table_instances.get_table_at(idx)
  }
}

impl Default for Store {
  fn default() -> Self {
    Store {
      function_instances: Vec::new(),
      function_types: Vec::new(),
      memory_instances: MemoryInstances::empty(),
      table_instances: TableInstances::empty(),
      global_instances: GlobalInstances::empty(),
    }
  }
}
