use super::decodable::{Decodable, Leb128Decodable, LimitDecodable, U32Decodable};
use alloc::vec::Vec;
use memory::Limit;
use trap::Result;

impl_decodable!(Section);
impl Leb128Decodable for Section {}
impl U32Decodable for Section {}
impl LimitDecodable for Section {}

impl Decodable for Section {
  type Item = Vec<Limit>;
  fn decode(&mut self) -> Result<Self::Item> {
    let count_of_section = self.decode_leb128_u32()?;
    (0..count_of_section)
      .map(|_| self.decode_limit())
      .collect::<Result<Vec<_>>>()
  }
}
