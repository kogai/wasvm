use super::decodable::Decodable;
use super::sec_element::{Element, ElementType};
use memory::Limit;
use std::{f32, f64};
use trap::Result;

#[derive(Debug)]
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
}

impl TableInstance {
  pub fn new(table: &TableType, element: &Element) -> Self {
    TableInstance {
      elements: element.init.to_owned(),
      max: match table.limit {
        Limit::NoUpperLimit(_) => None,
        Limit::HasUpperLimit(_, max) => Some(max),
      },
    }
  }
  pub fn len(&self) -> u32 {
    self.elements.len() as u32
  }
  pub fn get_function_address(&self, idx: u32) -> Option<u32> {
    self.elements.get(idx as usize).map(|x| *x)
  }
}

impl_decodable!(Section);
impl_decode_limit!(Section);

impl Decodable for Section {
  type Item = TableType;
  fn decode(&mut self) -> Result<Vec<Self::Item>> {
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
