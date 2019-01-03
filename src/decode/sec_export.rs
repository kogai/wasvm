use super::decodable::{Decodable, NameDecodable};
use alloc::string::String;
use alloc::vec::Vec;
use core::{f32, f64};
use module::{ExportDescriptor, ExternalInterface, ExternalInterfaces, ModuleDescriptor};
use trap::Result;

impl_decodable!(Section);
impl_name_decodable!(Section);

impl Decodable for Section {
  type Item = ExternalInterfaces;

  fn decode(&mut self) -> Result<Self::Item> {
    let count_of_section = self.decode_leb128_u32()?;
    let mut exports: ExternalInterfaces = ExternalInterfaces::new();
    for _ in 0..count_of_section {
      let name = self.decode_name()?;
      let export_descriptor = ExportDescriptor::from((self.next(), self.decode_leb128_u32()?));
      exports.insert(ExternalInterface::new(
        None,
        name,
        ModuleDescriptor::ExportDescriptor(export_descriptor),
      ));
    }
    Ok(exports)
  }
}
