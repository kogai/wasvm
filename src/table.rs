use alloc::string::String;
use alloc::vec::Vec;
use decode::{Element, TableType};
use memory::Limit;
use trap::{Result, Trap};

type FunctionAddress = u32;

#[derive(Debug, Clone)]
pub struct TableInstance {
  // FIXME: Replace with Vec<Option<FunctionAddress>>
  elements: Vec<FunctionAddress>,
  max: Option<u32>,
  pub export_name: Option<String>,
}

impl TableInstance {
  pub fn new(table: Option<&TableType>, element: Element, export_name: Option<String>) -> Self {
    TableInstance {
      elements: element.move_init_to(),
      max: match table {
        Some(TableType {
          limit: Limit::NoUpperLimit(_),
          ..
        }) => None,
        Some(TableType {
          limit: Limit::HasUpperLimit(_, max),
          ..
        }) => Some(*max),
        _ => None,
      },
      export_name,
    }
  }

  pub fn len(&self) -> usize {
    self.elements.len()
  }

  pub fn get_function_address(&self, idx: u32) -> Result<u32> {
    match self.elements.get(idx as usize) {
      Some(x) => Ok(*x),
      None => Err(Trap::UndefinedElement),
    }
  }
}
