use super::decodable::{Leb128Decodable, LimitDecodable, NewDecodable, U32Decodable};
use alloc::vec::Vec;
use error::{Result, WasmError};
use memory::Limit;

impl_decodable!(Section);
impl Leb128Decodable for Section {}
impl U32Decodable for Section {}
impl LimitDecodable for Section {}

impl NewDecodable for Section {
  type Item = Vec<Limit>;
  fn decode(&mut self) -> Result<Self::Item> {
    let count_of_section = self.decode_leb128_u32()?;
    (0..count_of_section)
      .map(|_| self.decode_limit().map_err(WasmError::Trap))
      .collect::<Result<Vec<_>>>()
  }
}
