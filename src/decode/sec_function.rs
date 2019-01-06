use super::decodable::{Decodable, U32Decodable};
use alloc::vec::Vec;
use trap::Result;

impl_decodable!(Section);
impl U32Decodable for Section {}

impl Decodable for Section {
  type Item = Vec<u32>;
  fn decode(&mut self) -> Result<Self::Item> {
    let count_of_section = self.decode_leb128_u32()?;
    (0..count_of_section)
      .map(|_| Ok(self.next()? as u32))
      .collect::<Result<Vec<_>>>()
  }
}
