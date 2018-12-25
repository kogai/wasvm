use super::code::ExportDescriptionCode;
use super::decodable::Decodable;
use std::collections::HashMap;
use std::{f32, f64};
use trap::Result;

impl_decodable!(Section);

#[derive(Eq, PartialEq, Hash, Debug)]
pub enum Export {
  Function,
  Global,
  Memory,
  Table,
}

pub type Exports = HashMap<Export, HashMap<String, usize>>;

impl Decodable for Section {
  type Item = Exports;

  fn decode(&mut self) -> Result<Self::Item> {
    let count_of_section = self.decode_leb128_u32()?;
    let mut function_map: HashMap<String, usize> = HashMap::new();
    let mut global_map: HashMap<String, usize> = HashMap::new();
    let mut memory_map: HashMap<String, usize> = HashMap::new();
    let mut table_map: HashMap<String, usize> = HashMap::new();
    let mut exports: Exports = HashMap::new();
    for _ in 0..count_of_section {
      let size_of_name = self.decode_leb128_u32()?;
      let mut buf = vec![];
      for _ in 0..size_of_name {
        buf.push(self.next()?);
      }
      let key = String::from_utf8(buf).expect("To encode export name has been failured.");
      let description_code = ExportDescriptionCode::from(self.next());
      let index = self.next()? as usize;
      match description_code {
        ExportDescriptionCode::ExportDescFunctionIdx => {
          function_map.insert(key, index);
        }
        ExportDescriptionCode::ExportDescGlobalIdx => {
          global_map.insert(key, index);
        }
        ExportDescriptionCode::ExportDescMemIdx => {
          memory_map.insert(key, index);
        }
        ExportDescriptionCode::ExportDescTableIdx => {
          table_map.insert(key, index);
        }
      };
    }
    exports.insert(Export::Function, function_map);
    exports.insert(Export::Global, global_map);
    exports.insert(Export::Memory, memory_map);
    exports.insert(Export::Table, table_map);
    Ok(exports)
  }
}
