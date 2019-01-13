use alloc::prelude::*;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::cmp::{Ordering, PartialOrd};
use core::fmt;
use core::mem::transmute;
use core::u32;
use decode::Data;
use global::GlobalInstances;
use inst::Inst;
use module::{ExternalInterface, ImportDescriptor, ModuleDescriptor};
use trap::{Result, Trap};
use value::Values;

// NOTE: 65536(64KiB) is constant data size per page.
const PAGE_SIZE: u32 = 65536;

// Prefer to rename MemoryType
#[derive(Clone, PartialEq)]
pub enum Limit {
  // (min)
  NoUpperLimit(u32),
  // (min, max)
  HasUpperLimit(u32, u32),
}

impl Limit {
  fn initial_min_size(&self) -> usize {
    let min_size = match self {
      Limit::NoUpperLimit(min) => min,
      Limit::HasUpperLimit(min, _) => min,
    };
    (PAGE_SIZE * min_size) as usize
  }
}
impl PartialOrd for Limit {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    use self::Limit::*;
    match (self, other) {
      // NOTE: In the mean of Limit, to compare min bound `A.min < B.min` represents self > other.
      // However, `A.max < B.max` self < other.
      // It seems the specification assumes to compare range of limitations.
      //
      // Example: in case of `A < B`.
      // +---+---+---+---+---+---+---+---+---+---+
      // | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
      // +---+---+---+---+---+---+---+---+---+---+
      //       ^       ^           ^       ^
      //     B.min    A.min ----- A.max   B.max
      (NoUpperLimit(min), NoUpperLimit(min_other)) => {
        if min > min_other {
          Some(Ordering::Less)
        } else if min == min_other {
          Some(Ordering::Equal)
        } else {
          Some(Ordering::Greater)
        }
      }
      (NoUpperLimit(min), HasUpperLimit(min_other, _)) => {
        if min > min_other {
          Some(Ordering::Less)
        } else {
          Some(Ordering::Greater)
        }
      }
      (HasUpperLimit(min, _), NoUpperLimit(min_other)) => {
        if min < min_other {
          Some(Ordering::Greater)
        } else {
          Some(Ordering::Less)
        }
      }
      (HasUpperLimit(min, max), HasUpperLimit(min_other, max_other)) => {
        if min > min_other || max < max_other {
          return Some(Ordering::Less);
        }
        if min < min_other || max > max_other {
          return Some(Ordering::Greater);
        }
        Some(Ordering::Equal)
      }
    }
  }
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
  data: Vec<u8>,
  limit: Limit,
  export_name: Option<String>,
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
  pub fn new(
    datas: Vec<Data>,
    limit: Limit,
    export_name: Option<String>,
    global_instances: &GlobalInstances,
  ) -> Result<Self> {
    let initial_size = limit.initial_min_size();
    let mut data = vec![0; initial_size];
    for Data { offset, init, .. } in datas.into_iter() {
      let offset = match offset.first() {
        Some(Inst::I32Const(offset)) => {
          if offset < &0 {
            return Err(Trap::DataSegmentDoesNotFit);
          }
          *offset
        }
        Some(Inst::GetGlobal(idx)) => global_instances.get_global_ext(*idx),
        x => unreachable!("Expected offset value of memory, got {:?}", x),
      } as usize;
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

  fn link(
    &mut self,
    datas: Vec<Data>,
    limit: Option<Limit>,
    global_instances: &GlobalInstances,
  ) -> Result<()> {
    if let Some(limit) = limit {
      self.limit = limit;
    };
    let data: &mut Vec<u8> = self.data.as_mut();
    for Data { offset, init, .. } in datas.into_iter() {
      let offset = match offset.first() {
        Some(Inst::I32Const(offset)) => *offset,
        Some(Inst::GetGlobal(idx)) => global_instances.get_global_ext(*idx),
        x => unreachable!("Expected offset value of memory, got {:?}", x),
      } as usize;
      for (i, d) in init.into_iter().enumerate() {
        data[i + offset] = d;
      }
    }
    Ok(())
  }

  fn validate(
    &mut self,
    datas: &Vec<Data>,
    limit: &Option<Limit>,
    global_instances: &GlobalInstances,
  ) -> Result<()> {
    let initial_size = match limit {
      Some(limit) => {
        let initial_size = limit.initial_min_size();
        initial_size
      }
      None => self.limit.initial_min_size(),
    };
    for Data { offset, init, .. } in datas.into_iter() {
      let offset = match offset.first() {
        Some(Inst::I32Const(offset)) => {
          if offset < &0 {
            return Err(Trap::DataSegmentDoesNotFit);
          }
          *offset
        }
        Some(Inst::GetGlobal(idx)) => global_instances.get_global_ext(*idx),
        x => unreachable!("Expected offset value of memory, got {:?}", x),
      } as usize;
      let size = offset + init.len();
      if size > initial_size {
        return Err(Trap::DataSegmentDoesNotFit);
      }
    }
    Ok(())
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

  pub fn limit_gt(&self, other_limit: &Limit) -> bool {
    &self.limit > other_limit
  }
}

impl fmt::Debug for MemoryInstance {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_struct("MemoryInstance")
      .field("export_name", &self.export_name)
      .field(
        "data",
        &self
          .data
          .iter()
          .filter(|d| d != &&0)
          .map(|d| format!("{}", d))
          .collect::<Vec<String>>()
          .join(", "),
      )
      .field("data.len()", &self.data.len())
      .field("limit", &self.limit)
      .finish()
  }
}

#[derive(Debug, Clone)]
pub struct MemoryInstances(Rc<RefCell<Vec<MemoryInstance>>>);

impl MemoryInstances {
  pub fn new(memory_instances: Vec<MemoryInstance>) -> Self {
    MemoryInstances(Rc::new(RefCell::new(memory_instances)))
  }

  pub fn empty() -> Self {
    MemoryInstances(Rc::new(RefCell::new(vec![])))
  }

  pub fn from(
    that: MemoryInstances,
    limit: Option<Limit>,
    datas: Vec<Data>,
    global_instances: &GlobalInstances,
  ) -> Result<Self> {
    that
      .0
      .borrow_mut()
      .get_mut(0)
      .ok_or(Trap::UnknownImport)?
      .link(datas, limit, global_instances)?;
    Ok(that.clone())
  }

  pub fn validate(
    that: &MemoryInstances,
    limit: &Option<Limit>,
    import: &ExternalInterface,
    datas: &Vec<Data>,
    global_instances: &GlobalInstances,
  ) -> Result<()> {
    that
      .0
      .borrow_mut()
      .get_mut(0)
      .ok_or(Trap::UnknownImport)
      .and_then(|instance| {
        if instance.export_name.as_ref() != Some(&import.name) {
          Err(Trap::UnknownImport)
        } else {
          Ok(instance)
        }
      })
      .and_then(|instance| match import {
        ExternalInterface {
          descriptor: ModuleDescriptor::ImportDescriptor(ImportDescriptor::Memory(limit)),
          ..
        } => {
          if instance.limit_gt(limit) {
            Err(Trap::IncompatibleImportType)
          } else {
            Ok(instance)
          }
        }
        x => unreachable!("Expected memory descriptor, got {:?}", x),
      })?
      .validate(datas, limit, global_instances)
  }

  pub fn data_size_small_than(&self, ptr: u32) -> bool {
    // FIXME: Use macro for commonize same borrowing procedure.
    self
      .0
      .borrow()
      .get(0)
      .expect("At least one memory instance expected")
      .data_size_smaller_than(ptr)
  }

  pub fn load_data_32(&self, from: u32, to: u32) -> u32 {
    self
      .0
      .borrow()
      .get(0)
      .expect("At least one memory instance expected")
      .load_data_32(from, to)
  }

  pub fn load_data_64(&self, from: u32, to: u32) -> u64 {
    self
      .0
      .borrow()
      .get(0)
      .expect("At least one memory instance expected")
      .load_data_64(from, to)
  }

  pub fn load_data_f32(&self, from: u32, to: u32) -> f32 {
    self
      .0
      .borrow()
      .get(0)
      .expect("At least one memory instance expected")
      .load_data_f32(from, to)
  }

  pub fn load_data_f64(&self, from: u32, to: u32) -> f64 {
    self
      .0
      .borrow()
      .get(0)
      .expect("At least one memory instance expected")
      .load_data_f64(from, to)
  }

  pub fn size_by_pages(&self) -> u32 {
    self
      .0
      .borrow()
      .get(0)
      .expect("At least one memory instance expected")
      .size_by_pages()
  }

  pub fn store_data(&self, from: u32, to: u32, value: Values) {
    self
      .0
      .borrow_mut()
      .get_mut(0)
      .expect("At least one memory instance expected")
      .store_data(from, to, value)
  }

  pub fn memory_grow(&self, increase_page: u32) -> Result<()> {
    self
      .0
      .borrow_mut()
      .get_mut(0)
      .expect("At least one memory instance expected")
      .memory_grow(increase_page)
  }

  pub fn clone_instance_by_name(&self, name: &String) -> Option<MemoryInstance> {
    let instance = self.0.borrow().get(0)?.clone();

    if instance.export_name == Some(name.to_owned()) {
      Some(instance)
    } else {
      None
    }
  }
}
