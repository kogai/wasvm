use decode::decodable::Decodable;
use std::{f32, f64};
use table::{ElementType, TableType};
use trap::{Result, Trap};

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
