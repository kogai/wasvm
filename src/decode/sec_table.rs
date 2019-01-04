use super::decodable::Decodable;
use super::sec_element::ElementType;
use alloc::vec::Vec;
use core::{f32, f64};
use memory::Limit;
use trap::{Result, Trap};

#[derive(Debug, Clone)]
pub struct TableType {
  element_type: ElementType,
  pub(crate) limit: Limit,
}

impl TableType {
  pub fn new(element_type: ElementType, limit: Limit) -> Self {
    TableType {
      element_type,
      limit,
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
