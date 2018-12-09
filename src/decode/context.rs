use function::FunctionInstance;
use global::GlobalInstance;
use memory::MemoryInstance;
use store::Store;
use table::TableInstance;
use trap::Result;

pub struct Context {
  function_instances: Vec<FunctionInstance>,
  memory_instances: Vec<MemoryInstance>,
  table_instances: Vec<TableInstance>,
  global_instances: Vec<GlobalInstance>,
}

impl Context {
  pub fn new(
    function_instances: Vec<FunctionInstance>,
    memory_instances: Vec<MemoryInstance>,
    table_instances: Vec<TableInstance>,
    global_instances: Vec<GlobalInstance>,
  ) -> Self {
    Context {
      function_instances,
      memory_instances,
      table_instances,
      global_instances,
    }
  }

  pub fn validate(&self) -> Result<Store> {
    unimplemented!();
  }
}
