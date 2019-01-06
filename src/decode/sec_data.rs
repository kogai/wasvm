use super::decodable::{Decodable, Peekable};
use alloc::vec::Vec;
use inst::Inst;
use trap::{Result, Trap};

#[derive(Debug)]
pub struct Data {
  pub memidx: u32,
  pub offset: Vec<Inst>,
  pub init: Vec<u8>,
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
impl Peekable for Section {}

impl Decodable for Section {
  type Item = Vec<Data>;
  fn decode(&mut self) -> Result<Self::Item> {
    let count_of_section = self.decode_leb128_u32()?;
    (0..count_of_section)
      .map(|_| {
        let memidx = self.decode_leb128_u32()?;
        let offset = self.decode_instructions()?;
        let size_of_data = self.decode_leb128_u32()?;
        let mut init = vec![];
        for _ in 0..size_of_data {
          init.push(self.next()?);
        }
        Ok(Data::new(memidx, offset, init))
      })
      .collect::<Result<Vec<_>>>()
  }
}
