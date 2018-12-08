use decode::decodable::Decodable;
use memory::Limit;
use std::{f32, f64};
use trap::Result;

impl_decodable!(Section);
impl_decode_limit!(Section);

impl Decodable for Section {
  type Item = Limit;
  fn decode(&mut self) -> Result<Vec<Self::Item>> {
    let count_of_section = self.decode_leb128_u32()?;
    (0..count_of_section)
      .map(|_| self.decode_limit())
      .collect::<Result<Vec<_>>>()
  }
}
