use inst::Instructions;

pub struct Element {
  table_idx: u32,
  offset: Instructions,
  init: Vec<u32>, // vec of funcidx
}

impl Element {
  pub fn new(table_idx: u32, offset: Instructions, init: Vec<u32>) -> Self {
    Element {
      table_idx,
      offset,
      init,
    }
  }
}
