use super::decodable::Decodable;
use element::Element;
use inst::Instructions;
use std::{f32, f64};
use trap::{Result, Trap};

impl_decodable!(Section);
impl_decode_code!(Section);

impl Section {
  fn decode_function_idx(&mut self) -> Result<Vec<u32>> {
    let count = self.decode_leb128_u32()?;
    Ok(
      (0..count)
        .map(|_| self.decode_leb128_u32().unwrap())
        .collect::<Vec<_>>(),
    )
  }
}

impl Decodable for Section {
  type Item = Element;
  fn decode(&mut self) -> Result<Vec<Self::Item>> {
    let count_of_section = self.decode_leb128_u32()?;
    (0..count_of_section)
      .map(|_| {
        let table_idx = self.decode_leb128_u32()?;
        let offset = self.decode_instructions()?;
        let init = self.decode_function_idx()?;
        Ok(Element::new(
          table_idx,
          Instructions::new(offset, vec![], vec![]),
          init,
        ))
      })
      .collect::<Result<Vec<_>>>()
  }
}
