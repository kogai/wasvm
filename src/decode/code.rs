#[derive(Debug)]
pub enum ExportDescriptionCode {
  ExportDescFunctionIdx,
  ExportDescTableIdx,
  ExportDescMemIdx,
  ExportDescGlobalIdx,
}

impl From<Option<u8>> for ExportDescriptionCode {
  fn from(code: Option<u8>) -> Self {
    use self::ExportDescriptionCode::*;
    match code {
      Some(0x00) => ExportDescFunctionIdx,
      Some(0x01) => ExportDescTableIdx,
      Some(0x02) => ExportDescMemIdx,
      Some(0x03) => ExportDescGlobalIdx,
      x => unreachable!("Export description code {:x?} does not supported yet.", x),
    }
  }
}
