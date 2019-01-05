use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::clone::Clone;
use decode::{Element, TableType};
use function::FunctionInstance;
use global::GlobalInstance;
use inst::Inst;
use memory::Limit;
use trap::{Result, Trap};
use value::Values;

#[derive(Debug, Clone)]
pub struct TableInstance {
  pub(crate) function_elements: Vec<Option<Rc<FunctionInstance>>>,
  pub(crate) export_name: Option<String>,
  table_type: TableType,
}

impl TableInstance {
  pub fn new(
    elements: Vec<Element>,
    table_type: TableType,
    export_name: Option<String>,
    global_instances: &Vec<GlobalInstance>,
    function_instances: &Vec<Rc<FunctionInstance>>,
  ) -> Result<Self> {
    let table_size = match table_type.limit {
      Limit::NoUpperLimit(min) | Limit::HasUpperLimit(min, _) => min,
    } as usize;
    let mut function_elements = vec![None; table_size];
    for el in elements.into_iter() {
      let offset = match el.offset.first() {
        Some(Inst::I32Const(offset)) => {
          if offset < &0 {
            return Err(Trap::ElementSegmentDoesNotFit);
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
      let mut function_addresses = el.wrap_by_option(function_instances);
      let end = offset + function_addresses.len();
      if end > function_elements.len() {
        return Err(Trap::ElementSegmentDoesNotFit);
      }
      function_addresses.swap_with_slice(&mut function_elements[offset..end]);
    }
    Ok(TableInstance {
      function_elements,
      export_name,
      table_type,
    })
  }

  pub fn len(&self) -> usize {
    self.function_elements.len()
  }

  pub fn get_function_instance(&self, idx: u32) -> Result<Rc<FunctionInstance>> {
    match self.function_elements.get(idx as usize) {
      Some(Some(x)) => Ok(x.clone()),
      Some(None) => Err(Trap::UninitializedElement),
      None => Err(Trap::UndefinedElement),
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
    TableInstances(Rc::new(RefCell::new(vec![])))
  }

  pub fn find_by_name(&self, name: &String) -> bool {
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

  pub fn get_table_at<'a>(&'a self, idx: u32) -> Option<TableInstance> {
    let table_instances = self.0.borrow();
    let table_instance = table_instances.get(idx as usize);
    table_instance.map(|x| x.clone())
  }

  pub fn update(
    &self,
    elements: Vec<Element>,
    global_instances: &Vec<GlobalInstance>,
    function_instances: &Vec<Rc<FunctionInstance>>,
  ) -> Result<()> {
    let mut table_instances = self.0.borrow_mut();
    let table_instance = table_instances.first_mut()?;
    let function_elements = &mut table_instance.function_elements;

    for el in elements.iter() {
      // FIXME: Commonize
      let offset = match el.offset.first() {
        Some(Inst::I32Const(offset)) => {
          if offset < &0 {
            return Err(Trap::ElementSegmentDoesNotFit);
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
      let mut function_addresses = el.wrap_by_option(function_instances);
      let end = offset + function_addresses.len();
      if end > function_elements.len() {
        return Err(Trap::ElementSegmentDoesNotFit);
      }
      function_addresses.swap_with_slice(&mut function_elements[offset..end]);
    }
    Ok(())
  }
}

impl Clone for TableInstances {
  fn clone(&self) -> Self {
    TableInstances(self.0.clone())
  }
}
