use inst::Inst;
use std::fmt;
use std::mem::transmute;
use value::Values;

// NOTE: 65536 is constant page size of webassembly.
const PAGE_SIZE: usize = 65536;

#[derive(Clone)]
pub enum Limit {
  NoUpperLimit(u32),
  HasUpperLimit(u32, u32),
}

impl fmt::Debug for Limit {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use self::Limit::*;
    write!(
      f,
      "{}",
      match self {
        NoUpperLimit(min) => format!("min:{}", min),
        HasUpperLimit(min, max) => format!("min:{},max:{}", min, max),
      }
    )
  }
}

#[derive(Debug)]
pub struct Data {
  pub memidx: u32,
  // FIXME: Offset may represents as u32?
  offset: Vec<Inst>,
  pub init: Vec<u8>,
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
  data: Vec<u8>, // Do not fixed size.
  limit: Limit,
}

macro_rules! impl_load_data {
  ($name: ident, $ty: ty, $value_ty:ty, $return_type: path) => {
    fn $name(&self, from: u32, to: u32) -> $value_ty {
      let data = &self.data[(from as usize)..(to as usize)];
      let width = data.len();
      let mut bit_buf: $ty = 0;
      for idx in 0..width {
        let bits = (data[idx] as $ty) << idx * 8;
        bit_buf = bit_buf ^ bits;
      }
      bit_buf as $value_ty
    }
  };
}

impl MemoryInstance {
  pub fn new(data: Data, limits: &Vec<Limit>) -> Self {
    let limit = limits.get(data.memidx as usize).expect("").to_owned();
    MemoryInstance {
      data: data.init,
      limit,
    }
  }
  pub fn size(&self) -> i32 {
    (self.data.len() / PAGE_SIZE) as i32
  }

  pub fn data_size_smaller_than(&self, ptr: u32) -> bool {
    ptr > (self.data.len()) as u32
  }

  impl_load_data!(load_data_i32, u32, i32, Values::I32);
  impl_load_data!(load_data_i64, u64, i64, Values::I64);
  impl_load_data!(load_data_f32, u32, u32, Values::F32);
  impl_load_data!(load_data_f64, u64, u64, Values::F64);

  pub fn load_data(&self, from: u32, to: u32, value_kind: &str) -> Values {
    match value_kind {
      "i32" => Values::I32(self.load_data_i32(from, to)),
      "i64" => Values::I64(self.load_data_i64(from, to)),
      "f32" => {
        let loaded = self.load_data_f32(from, to);
        Values::F32(f32::from_bits(loaded))
      }
      "f64" => {
        let loaded = self.load_data_f64(from, to);
        Values::F64(f64::from_bits(loaded))
      }
      _ => unreachable!(),
    }
  }
}

impl fmt::Debug for MemoryInstance {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "[{}] {:?}",
      self
        .data
        .iter()
        .map(|d| format!("{}", d))
        .collect::<Vec<String>>()
        .join(", "),
      self.limit
    )
  }
}
