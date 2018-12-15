use decode::decodable::Decodable;
use inst::Inst;
use std::{f32, f64};
use trap::{Result, Trap};

#[derive(Debug)]
pub struct Data {
  memidx: u32,
  // FIXME: Offset may represents as u32?
  offset: Vec<Inst>,
  init: Vec<u8>,
}

impl Data {
  pub fn new(memidx: u32, offset: Vec<Inst>, init: Vec<u8>) -> Self {
    Data {
      memidx,
      offset,
      init,
    }
  }
  pub fn get_data_idx(&self) -> u32 {
    self.memidx
  }
  pub fn get_init(self) -> Vec<u8> {
    self.init
  }
}

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
