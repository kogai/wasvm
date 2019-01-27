use core::convert::From;

#[derive(Debug, Clone, PartialEq)]
pub struct Indice(u32);

impl From<u32> for Indice {
  fn from(n: u32) -> Self {
    Indice(n)
  }
}

impl From<usize> for Indice {
  fn from(n: usize) -> Self {
    Indice(n as u32)
  }
}

impl Indice {
  pub fn to_usize(&self) -> usize {
    self.0 as usize
  }

  pub fn to_u32(&self) -> u32 {
    self.0
  }
}
