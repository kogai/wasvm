#[cfg(not(test))]
use alloc::prelude::*;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::clone::Clone;
use decode::{Element, TableType};
use error::{Result, Trap, WasmError};
use function::FunctionInstance;
use global::GlobalInstances;
use indice::Indice;
use isa::Isa;
use memory::Limit;

#[derive(Debug, Clone)]
pub struct TableInstance {
  pub(crate) function_elements: Vec<Option<FunctionInstance>>,
  pub(crate) export_name: Option<String>,
  table_type: TableType,
}

impl TableInstance {
  pub fn new(
    elements: Vec<Element>,
    table_type: TableType,
    export_name: Option<String>,
    global_instances: &GlobalInstances,
    function_instances: &[FunctionInstance],
  ) -> Result<Self> {
    let table_size = match table_type.limit {
      Limit::NoUpperLimit(min) | Limit::HasUpperLimit(min, _) => min,
    } as usize;
    let mut function_elements = vec![None; table_size];
    for el in elements.into_iter() {
      let offset = match Isa::from(*el.offset.first()?) {
        Isa::I32Const => {
          let mut buf = [0; 4];
          buf.clone_from_slice(&el.offset[1..5]);
          let offset = unsafe { core::mem::transmute::<_, u32>(buf) } as i32;
          if offset < 0 {
            return Err(WasmError::Trap(Trap::ElementSegmentDoesNotFit));
          }
          offset
        }
        Isa::GetGlobal => {
          let mut buf = [0; 4];
          buf.clone_from_slice(&el.offset[1..5]);
          let idx = Indice::from(unsafe { core::mem::transmute::<_, u32>(buf) });
          global_instances.get_global_ext(&idx)
        }
        x => unreachable!("Expected offset value of memory, got {:?}", x),
      } as usize;
      let mut function_addresses = el.wrap_by_option(function_instances);
      let end = offset + function_addresses.len();
      if end > function_elements.len() {
        return Err(WasmError::Trap(Trap::ElementSegmentDoesNotFit));
      }
      function_addresses.swap_with_slice(&mut function_elements[offset..end]);
    }
    Ok(TableInstance {
      function_elements,
      export_name,
      table_type,
    })
  }

  pub fn validate(
    elements: &[Element],
    table_type: &TableType,
    global_instances: &GlobalInstances,
    function_instances: &[FunctionInstance],
  ) -> Result<()> {
    let table_size = match table_type.limit {
      Limit::NoUpperLimit(min) | Limit::HasUpperLimit(min, _) => min,
    } as usize;
    for el in elements.iter() {
      let offset = match Isa::from(*el.offset.first()?) {
        Isa::I32Const => {
          let mut buf = [0; 4];
          buf.clone_from_slice(&el.offset[1..5]);
          let offset = unsafe { core::mem::transmute::<_, u32>(buf) } as i32;
          if offset < 0 {
            return Err(WasmError::Trap(Trap::ElementSegmentDoesNotFit));
          }
          offset
        }
        Isa::GetGlobal => {
          let mut buf = [0; 4];
          buf.clone_from_slice(&el.offset[1..5]);
          let idx = Indice::from(unsafe { core::mem::transmute::<_, u32>(buf) });
          global_instances.get_global_ext(&idx)
        }
        x => unreachable!("Expected offset value of memory, got {:?}", x),
      } as usize;
      let mut function_addresses = el.wrap_by_option(function_instances);
      let end = offset + function_addresses.len();
      if end > table_size {
        return Err(WasmError::Trap(Trap::ElementSegmentDoesNotFit));
      }
    }
    Ok(())
  }

  pub fn len(&self) -> usize {
    self.function_elements.len()
  }

  pub fn get_function_instance(&self, idx: u32) -> Result<FunctionInstance> {
    match self.function_elements.get(idx as usize) {
      Some(Some(x)) => Ok(x.clone()),
      Some(None) => Err(WasmError::Trap(Trap::UninitializedElement)),
      None => Err(WasmError::Trap(Trap::UndefinedElement)),
    }
  }
}

#[derive(Debug)]
pub struct TableInstances(Rc<RefCell<Vec<TableInstance>>>);

impl TableInstances {
  pub fn new(table_instances: Vec<TableInstance>) -> Self {
    TableInstances(Rc::new(RefCell::new(table_instances)))
  }

  pub fn empty() -> Self {
    TableInstances::new(vec![])
  }

  pub fn find_by_name(&self, name: &str) -> bool {
    match self.0.borrow().first() {
      Some(table_instance) => table_instance.export_name == Some(name.to_owned()),
      None => false,
    }
  }

  // NOTE: It represents `self.table_type > other_table_type`
  pub fn gt_table_type(&self, other: &TableType) -> bool {
    match self.0.borrow().first() {
      Some(table_instance) => &table_instance.table_type > other,
      None => false,
    }
  }

  pub fn get_table_at(&self, idx: &Indice) -> Option<TableInstance> {
    let table_instances = self.0.borrow();
    table_instances.get(idx.to_usize()).cloned()
  }

  pub fn link(
    &self,
    elements: &[Element],
    global_instances: &GlobalInstances,
    function_instances: &[FunctionInstance],
  ) -> Result<()> {
    let mut table_instances = self.0.borrow_mut();
    let table_instance = table_instances.first_mut()?;
    let function_elements = &mut table_instance.function_elements;

    for el in elements.iter() {
      let offset = match Isa::from(*el.offset.first()?) {
        Isa::I32Const => {
          let mut buf = [0; 4];
          buf.clone_from_slice(&el.offset[1..5]);
          unsafe { core::mem::transmute::<_, i32>(buf) }
        }
        Isa::GetGlobal => {
          let mut buf = [0; 4];
          buf.clone_from_slice(&el.offset[1..5]);
          let idx = Indice::from(unsafe { core::mem::transmute::<_, u32>(buf) });
          global_instances.get_global_ext(&idx)
        }
        x => unreachable!("Expected offset value of memory, got {:?}", x),
      } as usize;
      let mut function_addresses = el.wrap_by_option(function_instances);
      let end = offset + function_addresses.len();
      function_addresses.swap_with_slice(&mut function_elements[offset..end]);
    }
    Ok(())
  }

  pub fn validate(
    &self,
    elements: &[Element],
    global_instances: &GlobalInstances,
    function_instances: &[FunctionInstance],
  ) -> Result<()> {
    let mut table_instances = self.0.borrow_mut();
    let table_instance = table_instances.first_mut()?;
    let function_elements = &mut table_instance.function_elements;

    for el in elements.iter() {
      let offset = match Isa::from(*el.offset.first()?) {
        Isa::I32Const => {
          let mut buf = [0; 4];
          buf.clone_from_slice(&el.offset[1..5]);
          let offset = unsafe { core::mem::transmute::<_, u32>(buf) } as i32;
          if offset < 0 {
            return Err(WasmError::Trap(Trap::ElementSegmentDoesNotFit));
          }
          offset
        }
        Isa::GetGlobal => {
          let mut buf = [0; 4];
          buf.clone_from_slice(&el.offset[1..5]);
          let idx = Indice::from(unsafe { core::mem::transmute::<_, u32>(buf) });
          global_instances.get_global_ext(&idx)
        }
        x => unreachable!("Expected offset value of table, got {:?}", x),
      } as usize;
      let mut function_addresses = el.wrap_by_option(function_instances);
      let end = offset + function_addresses.len();
      if end > function_elements.len() {
        return Err(WasmError::Trap(Trap::ElementSegmentDoesNotFit));
      }
    }
    Ok(())
  }
}

impl Clone for TableInstances {
  fn clone(&self) -> Self {
    TableInstances(self.0.clone())
  }
}
