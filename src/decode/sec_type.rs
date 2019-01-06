use super::code::Code;
use super::decodable::Decodable;
use alloc::vec::Vec;
use core::convert::From;
use function::FunctionType;
use trap::{Result, Trap};
use value_type::ValueTypes;

impl_decodable!(Section);

impl Decodable for Section {
  type Item = Vec<FunctionType>;
  fn decode(&mut self) -> Result<Self::Item> {
    let count_of_type = self.decode_leb128_u32()?;
    (0..count_of_type)
      .map(|_| {
        let mut parameters = vec![];
        let mut returns = vec![];
        let _type_function = Code::from(self.next());
        let size_of_arity = self.decode_leb128_u32()?;
        for _ in 0..size_of_arity {
          parameters.push(ValueTypes::from(self.next()));
        }
        let size_of_result = self.decode_leb128_u32()?;
        for _ in 0..size_of_result {
          returns.push(ValueTypes::from(self.next()));
        }
        Ok(FunctionType::new(parameters, returns))
      })
      .collect::<Result<Vec<_>>>()
  }
}
