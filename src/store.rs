use function::FunctionInstance;
use memory::MemoryInstance;
use value::Values;

pub struct Store {
  function_instances: Vec<FunctionInstance>,
  memory_instance: MemoryInstance,
}

impl Store {
  pub fn new(function_instances: Vec<FunctionInstance>, memory_instance: MemoryInstance) -> Self {
    Store {
      function_instances,
      memory_instance,
    }
  }

  pub fn call(&self, fn_idx: usize) -> Option<&FunctionInstance> {
    self.function_instances.get(fn_idx)
  }

  pub fn get_function_idx(&self, invoke: &str) -> usize {
    self
      .function_instances
      .iter()
      .position(|f| f.find(invoke))
      .expect(&format!("Function [{}] did not found.", invoke))
  }

  #[cfg(test)]
  pub fn get_function_instance(&self) -> Vec<FunctionInstance> {
    self.function_instances.to_owned()
  }

  pub fn data_size_small_than(&self, ptr: u32) -> bool {
    self.memory_instance.data_size_smaller_than(ptr)
  }

  pub fn load_data(&self, from: u32, to: u32, value_kind: &str) -> Values {
    self.memory_instance.load_data(from, to, value_kind)
  }
}
