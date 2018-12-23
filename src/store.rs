use decode::TableInstance;
use function::{FunctionInstance, FunctionType};
use global::GlobalInstance;
use memory::MemoryInstance;
use std::rc::Rc;
use trap::Result;
use value::Values;
use value_type::ValueTypes;

#[derive(Debug)]
pub struct Store {
  function_instances: Vec<Rc<FunctionInstance>>,
  function_types: Vec<FunctionType>,
  memory_instances: Vec<MemoryInstance>,
  table_instances: Vec<TableInstance>,
  global_instances: Vec<GlobalInstance>,
}

impl Store {
  pub fn new(
    function_instances: Vec<Rc<FunctionInstance>>,
    function_types: Vec<FunctionType>,
    memory_instances: Vec<MemoryInstance>,
    table_instances: Vec<TableInstance>,
    global_instances: Vec<GlobalInstance>,
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
    self.function_instances.get(fn_idx).map(|x| x.clone())
  }

  pub fn get_function_type(&self, idx: u32) -> Option<&FunctionType> {
    self.function_types.get(idx as usize)
  }

  pub fn get_function_type_by_instance(&self, idx: u32) -> Option<FunctionType> {
    let function_type = self
      .get_function_instance(idx as usize)
      .map(|x| x.get_function_type());
    match function_type {
      Some(Ok(x)) => Some(x),
      _ => None,
    }
  }

  pub fn get_function_idx(&self, invoke: &str) -> usize {
    self
      .function_instances
      .iter()
      .position(|f| f.find(invoke))
      .expect(&format!("Function [{}] did not found.", invoke))
  }

  pub fn get_global(&mut self, idx: u32) -> Result<&Values> {
    let result = self
      .global_instances
      .get(idx as usize)
      .map(|g| g.get_value())?;
    Ok(result)
  }

  pub fn set_global(&mut self, idx: u32, value: Values) {
    self
      .global_instances
      .get_mut(idx as usize)
      .map(|g| g.set_value(value));
  }

  pub fn get_table_at(&self, idx: u32) -> Option<&TableInstance> {
    self.table_instances.get(idx as usize)
  }

  fn get_memory_instance(&self) -> &MemoryInstance {
    self
      .memory_instances
      .get(0)
      .expect("At least one memory instance expected")
  }
  fn get_mut_memory_instance(&mut self) -> &mut MemoryInstance {
    self
      .memory_instances
      .get_mut(0)
      .expect("At least one memory instance expected")
  }
  pub fn data_size_small_than(&self, ptr: u32) -> bool {
    self.get_memory_instance().data_size_smaller_than(ptr)
  }
  pub fn load_data(&self, from: u32, to: u32, value_kind: &ValueTypes) -> Values {
    self.get_memory_instance().load_data(from, to, value_kind)
  }
  pub fn store_data(&mut self, from: u32, to: u32, value: Values) {
    self.get_mut_memory_instance().store_data(from, to, value)
  }
  pub fn size_by_pages(&self) -> u32 {
    self.get_memory_instance().size_by_pages()
  }
  pub fn memory_grow(&mut self, increase_page: u32) -> Result<()> {
    self.get_mut_memory_instance().memory_grow(increase_page)
  }
}
