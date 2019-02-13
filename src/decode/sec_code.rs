use super::decodable::{
  Decodable, Leb128Decodable, Peekable, SignedIntegerDecodable, U32Decodable, U8Iterator,
};
use super::instruction::InstructionDecodable;
use alloc::vec::Vec;
use core::convert::From;
use error::runtime::Result;
use value_type::ValueTypes;

impl_decodable!(Section);

impl Peekable for Section {}
impl Leb128Decodable for Section {}
impl U32Decodable for Section {}
impl SignedIntegerDecodable for Section {}
impl InstructionDecodable for Section {}

impl Decodable for Section {
  type Item = Vec<Result<(Vec<u8>, Vec<ValueTypes>)>>;
  fn decode(&mut self) -> Result<Self::Item> {
    let count_of_section = self.decode_leb128_u32()?;
    (0..count_of_section)
      .map(|_| {
        let size_of_function = self.decode_leb128_u32()?;
        let end_of_function = self.byte_ptr + (size_of_function as usize);
        let count_of_locals = self.decode_leb128_u32()? as usize;
        let mut locals: Vec<ValueTypes> = Vec::with_capacity(count_of_locals);
        for _ in 0..count_of_locals {
          let count_of_type = self.decode_leb128_u32()?;
          let value_type = ValueTypes::from(self.next()?);
          for _ in 0..count_of_type {
            locals.push(value_type.clone());
          }
        }
        Ok(match self.decode_instructions() {
          Ok(expressions) => Ok((expressions, locals)),
          Err(err) => {
            self.byte_ptr = end_of_function;
            Err(err)
          }
        })
      })
      .collect::<Result<Vec<_>>>()
  }
}
