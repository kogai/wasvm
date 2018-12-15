use decode::Data;
use std::fmt;
use std::mem::transmute;
use trap::{Result, Trap};
use value::Values;

// NOTE: 65536 is constant page size of webassembly.
const PAGE_SIZE: u32 = 65536;

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

macro_rules! impl_store_data {
  ($name: ident, $length: expr, $ty: ty) => {
    fn $name (&mut self, v: $ty, from: u32, to: u32) {
        let bytes: [u8; $length] = unsafe { transmute(v) };
        for address in from..to {
          let idx = address - from;
          self.data[address as usize] = bytes[idx as usize];
        }
    }
  };
}

impl MemoryInstance {
  pub fn new(data: Data, limits: &Vec<Limit>) -> Self {
    let idx = data.get_data_idx();
    let limit = limits
      .get(idx as usize)
      .expect("Limitation of Data can't found.")
      .to_owned();
    let mut data = data.get_init();
    data.resize(PAGE_SIZE as usize, 0);
    MemoryInstance { data, limit }
  }
  fn data_size(&self) -> u32 {
    self.data.len() as u32
  }
  pub fn data_size_smaller_than(&self, ptr: u32) -> bool {
    ptr > self.data_size()
  }
  pub fn size_by_pages(&self) -> u32 {
    self.data_size() / PAGE_SIZE
  }
  pub fn memory_grow(&mut self, increase_page: u32) -> Result<()> {
    match self.limit {
      Limit::HasUpperLimit(_, max) if self.size_by_pages() + increase_page >= max => {
        return Err(Trap::FailToGrow)
      }
      _ => {
        let current_size = self.data.len();
        let growing_size = (increase_page * PAGE_SIZE) as usize;
        self.data.resize(current_size + growing_size, 0);
        Ok(())
      }
    }
  }

  impl_load_data!(load_data_i32, u32, i32, Values::I32);
  impl_load_data!(load_data_i64, u64, i64, Values::I64);
  impl_load_data!(load_data_f32, u32, u32, Values::F32);
  impl_load_data!(load_data_f64, u64, u64, Values::F64);
  impl_store_data!(store_data_i32, 4, i32);
  impl_store_data!(store_data_f32, 4, f32);
  impl_store_data!(store_data_i64, 8, i64);
  impl_store_data!(store_data_f64, 8, f64);

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
  pub fn store_data(&mut self, from: u32, to: u32, value: Values) {
    match value {
      Values::I32(v) => self.store_data_i32(v, from, to),
      Values::F32(v) => self.store_data_f32(v, from, to),
      Values::I64(v) => self.store_data_i64(v, from, to),
      Values::F64(v) => self.store_data_f64(v, from, to),
    };
  }
}

impl fmt::Debug for MemoryInstance {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "[{}]:{} {:?}",
      self
        .data
        .iter()
        .filter(|d| d != &&0)
        .map(|d| format!("{}", d))
        .collect::<Vec<String>>()
        .join(", "),
      self.data.len(),
      self.limit
    )
  }
}
