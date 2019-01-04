use alloc::string::String;
use alloc::vec::Vec;
use decode::{Element, TableType};
use global::GlobalInstance;
use inst::Inst;
use memory::Limit;
use trap::{Result, Trap};
use value::Values;

type FunctionAddress = u32;

#[derive(Debug, Clone)]
pub struct TableInstance {
  function_elements: Vec<Option<FunctionAddress>>,
  pub(crate) export_name: Option<String>,
}

impl TableInstance {
  pub fn new(
    elements: Vec<Element>,
    table_type: &TableType,
    export_name: Option<String>,
    global_instances: &Vec<GlobalInstance>,
  ) -> Result<Self> {
    let table_size = match table_type.limit {
      Limit::NoUpperLimit(min) | Limit::HasUpperLimit(min, _) => min,
    } as usize;
    let mut function_elements = vec![None; table_size];
    for el in elements.into_iter() {
      let offset = match el.offset.first() {
        Some(Inst::I32Const(offset)) => {
          if offset < &0 {
            return Err(Trap::InvalidElementSegment);
          }
          *offset
        }
        Some(Inst::GetGlobal(idx)) => global_instances
          .get(*idx as usize)
          .map(|g| match &g.value {
            Values::I32(ref v) => *v,
            x => unreachable!("Expect I32, got {:?}", x),
          })
          .expect(&format!(
            "Expect to get {:?} of {:?}, got None",
            idx, global_instances,
          )),
        x => unreachable!("Expected offset value of memory, got {:?}", x),
      } as usize;
      let mut function_addresses = el.wrap_by_option();
      let end = offset + function_addresses.len();
      function_addresses.swap_with_slice(&mut function_elements[offset..end]);
    }
    Ok(TableInstance {
      function_elements,
      export_name,
    })
  }

  pub fn len(&self) -> usize {
    self.function_elements.len()
  }

  pub fn get_function_address(&self, idx: u32) -> Result<u32> {
    match self.function_elements.get(idx as usize) {
      Some(Some(x)) => Ok(*x),
      Some(None) => Err(Trap::UndefinedElement),
      None => unreachable!(),
    }
  }
}
