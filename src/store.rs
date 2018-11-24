use function::FunctionInstance;
use memory::MemoryInstance;

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
}
