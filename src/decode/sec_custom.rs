use super::decodable::{Leb128Decodable, NameDecodable, Decodable, U32Decodable, U8Iterator};
use alloc::string::String;
use alloc::vec::Vec;
use error::Result;

impl_decodable!(Section);
impl Leb128Decodable for Section {}
impl U32Decodable for Section {}
impl NameDecodable for Section {}

impl Decodable for Section {
  type Item = Vec<(String, Vec<u8>)>;

  fn decode(&mut self) -> Result<Self::Item> {
    let key = self.decode_name()?;
    let mut codes = vec![];
    while let Some(code) = self.next() {
      codes.push(code);
    }
    Ok(vec![(key, codes)])
  }
}
