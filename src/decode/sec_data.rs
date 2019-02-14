use super::decodable::{
  Leb128Decodable, NewDecodable, Peekable, SignedIntegerDecodable, U32Decodable, U8Iterator,
};
use super::instruction::InstructionDecodable;
use alloc::vec::Vec;
use error::Result;

#[derive(Debug)]
pub struct Data {
  pub memidx: u32,
  pub offset: Vec<u8>,
  pub init: Vec<u8>,
}

impl Data {
  pub fn new(memidx: u32, offset: Vec<u8>, init: Vec<u8>) -> Self {
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
impl Peekable for Section {}
impl Leb128Decodable for Section {}
impl U32Decodable for Section {}
impl SignedIntegerDecodable for Section {}
impl InstructionDecodable for Section {}

impl NewDecodable for Section {
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
