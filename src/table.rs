use memory::Limit;

pub struct ElementType;

impl From<Option<u8>> for ElementType {
  fn from(code: Option<u8>) -> Self {
    match code {
      Some(0x70) => ElementType,
      x => unreachable!("Expected element-type code, got {:?}", x),
    }
  }
}

pub struct TableInstance {
  // elem,
  // max,
  element_type: ElementType,
  limit: Limit,
}

impl TableInstance {
  pub fn new(element_type: ElementType, limit: Limit) -> Self {
    TableInstance {
      element_type,
      limit,
    }
  }
}
