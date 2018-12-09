use function::FunctionInstance;
use global::GlobalInstance;
use memory::MemoryInstance;
use store::Store;
use table::TableInstance;
use trap::{Result, Trap};

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
    Err(Trap::TypeMismatch)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use code::ValueTypes;
  use function::FunctionType;
  use inst::Inst;

  #[test]
  fn test_validate_return_type() {
    let export_name = None;
    let function_type = Ok(FunctionType::new(vec![], vec![ValueTypes::I32]));
    let locals = vec![];
    let type_idx = 0;
    let body = vec![Inst::I64Const(0), Inst::End];
    let actual = Context::new(
      vec![FunctionInstance::new(
        export_name,
        function_type,
        locals,
        type_idx,
        body,
      )],
      vec![],
      vec![],
      vec![],
    )
    .validate();
    assert_eq!(actual.unwrap_err(), Trap::TypeMismatch);
  }
}
