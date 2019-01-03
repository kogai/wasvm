use super::decodable::Decodable;
use super::sec_element::{Element, ElementType};
use alloc::string::String;
use alloc::vec::Vec;
use core::{f32, f64};
use memory::Limit;
use trap::{Result, Trap};

#[derive(Debug, Clone)]
pub struct TableType {
  element_type: ElementType,
  limit: Limit,
}

impl TableType {
  pub fn new(element_type: ElementType, limit: Limit) -> Self {
    TableType {
      element_type,
      limit,
    }
  }
}

#[derive(Debug, Clone)]
pub struct TableInstance {
  elements: Vec<u32>, // Vec of function address
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

impl_decodable!(Section);
impl_decode_limit!(Section);

impl Decodable for Section {
  type Item = Vec<TableType>;
  fn decode(&mut self) -> Result<Self::Item> {
    let count_of_section = self.decode_leb128_u32()?;
    (0..count_of_section)
      .map(|_| {
        let element_type = ElementType::from(self.next());
        let limit = self.decode_limit()?;
        Ok(TableType::new(element_type, limit))
      })
      .collect::<Result<Vec<_>>>()
  }
}
