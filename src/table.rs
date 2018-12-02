use element::Element;
use memory::Limit;

#[derive(Debug)]
pub enum ElementType {
  AnyFunc,
}

impl From<Option<u8>> for ElementType {
  fn from(code: Option<u8>) -> Self {
    match code {
      Some(0x70) => ElementType::AnyFunc,
      x => unreachable!("Expected element-type code, got {:?}", x),
    }
  }
}

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

#[derive(Debug)]
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
}
