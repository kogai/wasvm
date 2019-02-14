use super::decodable::{
  Leb128Decodable, LimitDecodable, NameDecodable, NewDecodable, U32Decodable, U8Iterator,
};
use super::sec_element::ElementType;
use super::sec_table::TableType;
use alloc::vec::Vec;
use error::Result;
use global::GlobalType;
use module::{
  ExternalInterface, ExternalInterfaces, ImportDescriptor, ModuleDescriptor, FUNCTION_DESCRIPTOR,
  GLOBAL_DESCRIPTOR, MEMORY_DESCRIPTOR, TABLE_DESCRIPTOR,
};
use value_type::ValueTypes;

impl_decodable!(Section);
impl Leb128Decodable for Section {}
impl U32Decodable for Section {}
impl LimitDecodable for Section {}
impl NameDecodable for Section {}

impl NewDecodable for Section {
  type Item = ExternalInterfaces;
  fn decode(&mut self) -> Result<Self::Item> {
    let count_of_section = self.decode_leb128_u32()?;
    let mut imports: ExternalInterfaces = ExternalInterfaces::default();
    for _ in 0..count_of_section {
      let module_name = self.decode_name()?;
      let name = self.decode_name()?;
      let import_descriptor = match From::from(self.next()) {
        FUNCTION_DESCRIPTOR => ImportDescriptor::Function(From::from(self.decode_leb128_u32()?)),
        TABLE_DESCRIPTOR => ImportDescriptor::Table(TableType::new(
          ElementType::from(self.next()),
          self.decode_limit()?,
        )),
        MEMORY_DESCRIPTOR => ImportDescriptor::Memory(self.decode_limit()?),
        GLOBAL_DESCRIPTOR => {
          let value_type = ValueTypes::from(self.next()?);
          let global_type = GlobalType::new(self.next(), value_type)?;
          ImportDescriptor::Global(global_type)
        }
      };
      imports.push(ExternalInterface::new(
        Some(module_name),
        name,
        ModuleDescriptor::ImportDescriptor(import_descriptor),
      ));
    }
    Ok(imports)
  }
}
