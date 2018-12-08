use code::ExportDescriptionCode;
use decode::decodable::Decodable;
use std::convert::From;
use std::{f32, f64};
use trap::{Result, Trap};

impl_decodable!(Section);

impl Decodable for Section {
  type Item = (String, usize);
  fn decode(&mut self) -> Result<Vec<Self::Item>> {
    let count_of_section = self.decode_leb128_u32()?;
    (0..count_of_section)
      .map(|_| {
        let size_of_name = self.decode_leb128_u32()?;
        let mut buf = vec![];
        for _ in 0..size_of_name {
          buf.push(self.next()?);
        }
        let key = String::from_utf8(buf).expect("To encode export name has been failured.");
        let idx_of_fn = match ExportDescriptionCode::from(self.next()) {
          ExportDescriptionCode::ExportDescFunctionIdx => self.next()?,
          x => unimplemented!("{:?}", x),
        };
        Ok((key, idx_of_fn as usize))
      })
      .collect::<Result<Vec<_>>>()
  }
}
