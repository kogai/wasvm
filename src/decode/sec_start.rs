use super::decodable::{Decodable, Leb128Decodable, U32Decodable};
use alloc::vec::Vec;
use trap::Result;

impl_decodable!(Section);
impl Leb128Decodable for Section {}
impl U32Decodable for Section {}

impl Decodable for Section {
  type Item = u32;
  fn decode(&mut self) -> Result<Self::Item> {
    let start_fn_idx = self.decode_leb128_u32()?;
    Ok(start_fn_idx)
  }
}
