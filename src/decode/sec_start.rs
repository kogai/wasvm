use super::decodable::Decodable;
use core::{f32, f64};
use trap::Result;

impl_decodable!(Section);

impl Decodable for Section {
  type Item = u32;
  fn decode(&mut self) -> Result<Self::Item> {
    let start_fn_idx = self.decode_leb128_u32()?;
    Ok(start_fn_idx)
  }
}
