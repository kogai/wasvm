use inst::Inst;
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
macro_rules! impl_load_data {
  ($name: ident, $ty: ty, $value_ty:ty, $return_type: path) => {
    fn $name(&self, from: u32, to: u32) -> Values {
      let data = &self.data[(from as usize)..(to as usize)];
      let width = data.len();
      let mut bit_buf: $ty = 0;
      for idx in 0..width {
        let bits = (data[idx] as $ty) << idx * 8;
        bit_buf = bit_buf ^ bits;
      }
      $return_type(bit_buf as $value_ty)
    }
  };
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
    ptr > (self.data.len()) as u32
  }

  impl_load_data!(load_data_i32, u32, i32, Values::I32);
  impl_load_data!(load_data_i64, u64, i64, Values::I64);

  pub fn load_data(&self, from: u32, to: u32, value_kind: &str) -> Values {
    match value_kind {
      "i32" => self.load_data_i32(from, to),
      "i64" => self.load_data_i64(from, to),
      _ => unreachable!(),
    }
  }
}
