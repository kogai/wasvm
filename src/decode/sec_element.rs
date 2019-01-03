use super::decodable::Decodable;
use alloc::vec::Vec;
use core::{f32, f64};
use inst::Inst;
use trap::{Result, Trap};

#[derive(Debug)]
pub struct Element {
  table_idx: u32,
  offset: Vec<Inst>,
  init: Vec<u32>, // vec of funcidx
}

impl Element {
  pub fn new(table_idx: u32, offset: Vec<Inst>, init: Vec<u32>) -> Self {
    Element {
      table_idx,
      offset,
      init,
    }
  }
  pub fn get_table_idx(&self) -> usize {
    self.table_idx as usize
  }
  pub fn move_init_to(self) -> Vec<u32> {
    self.init
  }
}

#[derive(Debug, Clone)]
pub enum ElementType {
  AnyFunc,
}

impl From<Option<u8>> for ElementType {
  fn from(code: Option<u8>) -> Self {
    match code {
      Some(0x70) => ElementType::AnyFunc,
      x => unreachable!("Expected element-type code, got {:?}", x),
    }
  }
}

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
  type Item = Vec<Element>;
  fn decode(&mut self) -> Result<Self::Item> {
    let count_of_section = self.decode_leb128_u32()?;
    (0..count_of_section)
      .map(|_| {
        let table_idx = self.decode_leb128_u32()?;
        let offset = self.decode_instructions()?;
        let init = self.decode_function_idx()?;
        Ok(Element::new(table_idx, offset, init))
      })
      .collect::<Result<Vec<_>>>()
  }
}
