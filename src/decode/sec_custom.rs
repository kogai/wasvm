use super::decodable::{Decodable, NameDecodable};
use alloc::string::String;
use alloc::vec::Vec;
use trap::{Result, Trap};

impl_decodable!(Section);
impl_name_decodable!(Section);

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
