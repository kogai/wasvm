use decode::decodable::Decodable;
use memory::Data;
use std::{f32, f64};
use trap::{Result, Trap};

impl_decodable!(Section);
impl_decode_code!(Section);

impl Decodable for Section {
  type Item = Data;
  fn decode(&mut self) -> Result<Vec<Self::Item>> {
    let count_of_section = self.decode_leb128_u32()?;
    (0..count_of_section)
      .map(|_| {
        let memidx = self.decode_leb128_u32()? as u32;
        let offset = self.decode_instructions()?;
        let mut size_of_data = self.next()?;
        let mut init = vec![];
        while size_of_data != 0 {
          size_of_data -= 1;
          init.push(self.next()?);
        }
        Ok(Data::new(memidx, offset, init))
      })
      .collect::<Result<Vec<_>>>()
  }
}
