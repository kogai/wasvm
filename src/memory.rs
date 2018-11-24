use inst::Inst;

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
}
