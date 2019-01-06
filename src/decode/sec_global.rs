use super::decodable::{Decodable, Peekable};
use alloc::vec::Vec;
use global::GlobalType;
use trap::{Result, Trap};
use value::Values;
use value_type::ValueTypes;

impl_decodable!(Section);
impl_decode_code!(Section);
impl Peekable for Section {}

impl Decodable for Section {
  type Item = Vec<(GlobalType, Values)>;
  fn decode(&mut self) -> Result<Self::Item> {
    let count_of_section = self.decode_leb128_u32()?;
    (0..count_of_section)
      .map(|_| {
        let value_type = ValueTypes::from(self.next());
        let global_type = GlobalType::new(self.next(), value_type);
        let init = self.decode_instructions()?;
        let value = init.first()?.get_value_ext();
        Ok((global_type, value))
      })
      .collect::<Result<Vec<_>>>()
  }
}
