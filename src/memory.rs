use inst::Inst;
use std::mem::transmute;
use value::Values;

const MEMORY_MAX: usize = 65536;

#[derive(Debug)]
pub enum Memory {
  NoUpperLimit(u32),
  HasUpperLimit(u32, u32),
}

#[derive(Debug)]
pub struct Data {
  memidx: u32,
  offset: Vec<Inst>,
  init: Vec<u8>,
}

impl Data {
  pub fn new(memidx: u32, offset: Vec<Inst>, init: Vec<u8>) -> Self {
    Data {
      memidx,
      offset,
      init,
    }
  }
}

pub struct MemoryInstance {
  data: [u8; MEMORY_MAX],
}

impl MemoryInstance {
  pub fn new(datas: Vec<Data>) -> Self {
    let bytes = datas
      .into_iter()
      .map(|data| data.init)
      .flatten()
      .enumerate();
    let mut data = [0; MEMORY_MAX];
    for (idx, byte) in bytes {
      data[idx] = byte;
    }
    MemoryInstance { data }
  }

  pub fn data_size_smaller_than(&self, ptr: u32) -> bool {
    ptr > (self.data.len() - 1) as u32
  }

  pub fn load_data(&self, from: u32, to: u32, is_signed: bool) -> Values {
    let data = &self.data[(from as usize)..(to as usize)];
    let width = data.len();
    let mut bit_buf: u32 = 0;
    for idx in 0..width {
      let bits = (data[idx] as u32) << idx * 8;
      bit_buf = bit_buf ^ bits;
    }
    Values::I32(bit_buf as i32)
  }
}
