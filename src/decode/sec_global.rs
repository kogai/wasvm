use super::decodable::{
  Decodable, Leb128Decodable, Peekable, SignedIntegerDecodable, U32Decodable, U8Iterator,
};
use super::instruction::InstructionDecodable;
use alloc::vec::Vec;
use error::Result;
use global::GlobalType;
use value_type::ValueTypes;

impl_decodable!(Section);
impl Peekable for Section {}
impl Leb128Decodable for Section {}
impl U32Decodable for Section {}
impl SignedIntegerDecodable for Section {}
impl InstructionDecodable for Section {}

impl Decodable for Section {
  type Item = Vec<(GlobalType, Vec<u8>)>;
  fn decode(&mut self) -> Result<Self::Item> {
    let count_of_section = self.decode_leb128_u32()?;
    (0..count_of_section)
      .map(|_| {
        let value_type = ValueTypes::from(self.next()?);
        let global_type = GlobalType::new(self.next(), value_type)?;
        let init = self.decode_instructions()?;
        Ok((global_type, init))
      })
      .collect::<Result<Vec<_>>>()
  }
}
