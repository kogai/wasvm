use super::decodable::{Decodable, NameDecodable};
use std::{f32, f64};
use trap::Result;

#[derive(Debug)]
pub enum ImportDescriptor {
  Type(u32),
  Table(u32),
  Memory(u32),
  Global(u32),
}

impl From<(Option<u8>, u32)> for ImportDescriptor {
  fn from(codes: (Option<u8>, u32)) -> Self {
    use self::ImportDescriptor::*;
    match codes.0 {
      Some(0x0) => Type(codes.1),
      Some(0x1) => Table(codes.1),
      Some(0x2) => Memory(codes.1),
      Some(0x3) => Global(codes.1),
      x => unreachable!("Expected import descriptor, got {:?}", x),
    }
  }
}

#[derive(Debug)]
pub struct Import {
  module_name: String,
  name: String,
  import_descriptor: ImportDescriptor,
}

impl_decodable!(Section);
impl_name_decodable!(Section);

impl Decodable for Section {
  type Item = Vec<Import>;
  fn decode(&mut self) -> Result<Self::Item> {
    let count_of_section = self.decode_leb128_u32()?;
    (0..count_of_section)
      .map(|_| {
        let module_name = self.decode_name()?;
        let name = self.decode_name()?;
        let import_descriptor = ImportDescriptor::from((self.next(), self.decode_leb128_u32()?));
        Ok(Import {
          module_name,
          name,
          import_descriptor,
        })
      })
      .collect::<Result<Vec<_>>>()
  }
}
