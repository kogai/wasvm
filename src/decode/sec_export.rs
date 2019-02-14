use super::decodable::{Leb128Decodable, NameDecodable, NewDecodable, U32Decodable, U8Iterator};
use alloc::vec::Vec;
use error::Result;
use module::{ExportDescriptor, ExternalInterface, ExternalInterfaces, ModuleDescriptor};

impl_decodable!(Section);
impl Leb128Decodable for Section {}
impl U32Decodable for Section {}
impl NameDecodable for Section {}

impl NewDecodable for Section {
  type Item = ExternalInterfaces;

  fn decode(&mut self) -> Result<Self::Item> {
    let count_of_section = self.decode_leb128_u32()?;
    let mut exports: ExternalInterfaces = ExternalInterfaces::default();
    for _ in 0..count_of_section {
      let name = self.decode_name()?;
      let export_descriptor = ExportDescriptor::from((self.next(), self.decode_leb128_u32()?));
      exports.push(ExternalInterface::new(
        None,
        name,
        ModuleDescriptor::ExportDescriptor(export_descriptor),
      ));
    }
    Ok(exports)
  }
}
