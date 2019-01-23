use super::decodable::{
  Decodable, Leb128Decodable, Peekable, SignedIntegerDecodable, U32Decodable,
};
use super::instruction::InstructionDecodable;
use alloc::vec::Vec;
use function::FunctionInstance;
use inst::{Indice, Inst};
use trap::Result;

#[derive(Debug, Clone)]
pub struct Element {
  pub(crate) table_idx: Indice,
  pub(crate) offset: Vec<Inst>,
  pub(crate) init: Vec<Indice>, // FIXME: Vec of funcidx, Use Indice type
}

impl Element {
  pub fn new(table_idx: Indice, offset: Vec<Inst>, init: Vec<Indice>) -> Self {
    Element {
      table_idx,
      offset,
      init,
    }
  }

  pub fn get_table_idx(&self) -> &Indice {
    &self.table_idx
  }

  pub fn move_init_to(&self) -> Vec<Indice> {
    self.init.clone()
  }

  pub(crate) fn wrap_by_option(
    &self,
    function_instances: &[FunctionInstance],
  ) -> Vec<Option<FunctionInstance>> {
    self
      .init
      .iter()
      .map(|fn_idx| function_instances.get(fn_idx.to_usize()).cloned())
      .collect()
  }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
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
impl Peekable for Section {}
impl Leb128Decodable for Section {}
impl U32Decodable for Section {}
impl SignedIntegerDecodable for Section {}
impl InstructionDecodable for Section {}

impl Section {
  fn decode_function_idx(&mut self) -> Result<Vec<Indice>> {
    let count = self.decode_leb128_u32()?;
    (0..count)
      .map(|_| self.decode_leb128_u32().map(From::from))
      .collect::<Result<Vec<_>>>()
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
        Ok(Element::new(From::from(table_idx), offset, init))
      })
      .collect::<Result<Vec<_>>>()
  }
}
