use decode::decodable::Decodable;
use std::{f32, f64};
use trap::{Result, Trap};

impl_decodable!(Section);

impl Decodable<u32> for Section {
  fn decode(&mut self) -> Result<Vec<u32>> {
    let count_of_section = self.decode_leb128_u32()?;
    (0..count_of_section)
      .map(|_| Ok(self.next()? as u32))
      .collect::<Result<Vec<_>>>()
  }
}
