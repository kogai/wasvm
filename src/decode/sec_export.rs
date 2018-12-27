use super::code::ExportDescriptionCode;
use super::decodable::{Decodable, NameDecodable};
use std::collections::HashMap;
use std::{f32, f64};
use trap::Result;

impl_decodable!(Section);
impl_name_decodable!(Section);

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub enum Export {
  Function,
  Global,
  Memory,
  Table,
}

pub type Exports = HashMap<String, (Export, usize)>;

impl Decodable for Section {
  type Item = Exports;

  fn decode(&mut self) -> Result<Self::Item> {
    let count_of_section = self.decode_leb128_u32()?;
    let mut exports: Exports = HashMap::new();
    for _ in 0..count_of_section {
      let key = self.decode_name()?;
      let description_code = ExportDescriptionCode::from(self.next());
      let index = self.next()? as usize;
      let kind = match description_code {
        ExportDescriptionCode::ExportDescFunctionIdx => Export::Function,
        ExportDescriptionCode::ExportDescGlobalIdx => Export::Global,
        ExportDescriptionCode::ExportDescMemIdx => Export::Memory,
        ExportDescriptionCode::ExportDescTableIdx => Export::Table,
      };
      exports.insert(key, (kind, index));
    }
    Ok(exports)
  }
}
