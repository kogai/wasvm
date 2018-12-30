use decode::Data;
use inst::Inst;
use std::fmt;
use std::mem::transmute;
use std::u32;
use trap::{Result, Trap};
use value::Values;

// NOTE: 65536(64KiB) is constant data size per page.
const PAGE_SIZE: u32 = 65536;

// Prefer to rename MemoryType
#[derive(Clone)]
pub enum Limit {
  // (min)
  NoUpperLimit(u32),
  // (min, max)
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

#[derive(Clone)]
pub struct MemoryInstance {
  data: Vec<u8>, // Do not fixed size.
  limit: Limit,
  pub export_name: Option<String>,
}

macro_rules! impl_load_data {
  ($name: ident, $ty: ty) => {
    pub fn $name(&self, from: u32, to: u32) -> $ty {
      let data = &self.data[(from as usize)..(to as usize)];
      let mut bit_buf: $ty = 0;
      for (idx, d) in data.iter().enumerate() {
        let bits = (*d as $ty) << idx * 8;
        bit_buf = bit_buf ^ bits;
      }
      bit_buf
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
  pub fn new(datas: Vec<Data>, limit: Limit, export_name: Option<String>) -> Result<Self> {
    let min_size = match limit {
      Limit::NoUpperLimit(min) => min,
      Limit::HasUpperLimit(min, _) => min,
    };
    let initial_size = (PAGE_SIZE * min_size) as usize;
    let mut data = vec![0; initial_size];
    for Data { offset, init, .. } in datas.into_iter() {
      let offset = match offset.first() {
        Some(Inst::I32Const(offset)) => *offset as usize,
        x => unreachable!("Expected offset value of memory, got {:?}", x),
      };
      let size = offset + init.len();
      if size > initial_size {
        return Err(Trap::DataSegmentDoesNotFit);
      }
      for (i, d) in init.into_iter().enumerate() {
        data[i + offset] = d;
      }
    }

    Ok(MemoryInstance {
      data,
      limit,
      export_name,
    })
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
      Limit::HasUpperLimit(_, max) if self.size_by_pages() + increase_page > max => {
        return Err(Trap::FailToGrow)
      }
      _ => {
        let current_size = self.data.len() as u32;
        match increase_page.checked_mul(PAGE_SIZE) {
          Some(growing_size) => match current_size.checked_add(growing_size) {
            Some(next_size) => {
              self.data.resize(next_size as usize, 0);
              Ok(())
            }
            None => Err(Trap::FailToGrow),
          },
          None => Err(Trap::FailToGrow),
        }
      }
    }
  }

  impl_load_data!(load_data_32, u32);
  impl_load_data!(load_data_64, u64);

  pub fn load_data_f32(&self, from: u32, to: u32) -> f32 {
    f32::from_bits(self.load_data_32(from, to))
  }

  pub fn load_data_f64(&self, from: u32, to: u32) -> f64 {
    f64::from_bits(self.load_data_64(from, to))
  }

  impl_store_data!(store_data_i32, 4, i32);
  impl_store_data!(store_data_f32, 4, f32);
  impl_store_data!(store_data_i64, 8, i64);
  impl_store_data!(store_data_f64, 8, f64);

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
