use code::{Code, ValueTypes};
use decode::decodable::Decodable;
use function::FunctionType;
use std::convert::From;
use std::{f32, f64};
use trap::{Result, Trap};

impl_decodable!(SectionType);

impl Decodable<FunctionType> for SectionType {
  fn decode(&mut self) -> Result<Vec<FunctionType>> {
    let count_of_type = self.decode_leb128_u32()?;
    let xs: Result<Vec<FunctionType>> = (0..count_of_type)
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
      .collect::<Result<Vec<_>>>();
    xs
  }
}
