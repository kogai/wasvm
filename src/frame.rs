use alloc::vec::Vec;
use core::cell::{Cell, RefCell, RefMut};
use core::fmt;
use core::ops::Sub;
use function::FunctionInstance;
use indice::Indice;
use stack::StackEntry;
use trap::Result;
use value_type::ValueTypes;

macro_rules! impl_pop_bytes {
  ($name: ident, $ty: ty, $width: expr) => {
    pub(crate) fn $name(&self) -> Result<$ty> {
      let mut buf = [0; $width];
      let body = self.function_instance.body();
      let start = self.ptr.get() as usize;
      let end = start + $width;
      buf.clone_from_slice(&body[start..end]);
      self.ptr.set((start + $width) as u32);
      Ok(unsafe { core::mem::transmute::<_, $ty>(buf) })
    }
  };
}

#[derive(PartialEq)]
pub struct Frame {
  // FIXME: No need to hold local_variables in frame.
  local_variables: RefCell<Vec<StackEntry>>,
  pub(crate) function_instance: FunctionInstance,
  ptr: Cell<u32>,
  pub last_ptr: u32,
  pub return_ptr: usize,
  pub prev_return_ptr: usize,
}

impl Frame {
  impl_pop_bytes!(pop_raw_u32, u32, 4);
  impl_pop_bytes!(pop_raw_u64, u64, 8);

  pub fn new(
    return_ptr: usize,
    prev_return_ptr: usize,
    function_instance: FunctionInstance,
    arguments: &mut Vec<StackEntry>,
  ) -> Self {
    let last_ptr = function_instance.get_expressions_count() as u32;
    Frame {
      local_variables: Frame::derive_local_variables(
        arguments,
        function_instance.local_variables(),
      ),
      function_instance,
      last_ptr,
      return_ptr,
      prev_return_ptr,
      ptr: Cell::new(0),
    }
  }

  pub fn is_completed(&self) -> bool {
    self.ptr.get().ge(&self.last_ptr)
  }

  pub fn is_fresh(&self) -> bool {
    self.ptr.get().eq(&0)
  }

  pub fn get_local_variables(&self) -> RefMut<Vec<StackEntry>> {
    self.local_variables.borrow_mut()
  }

  // From: args[2,1]; locals[4,3]
  // To [4,3,2,1]
  fn derive_local_variables(
    arguments: &mut Vec<StackEntry>,
    mut local_variables: Vec<StackEntry>,
  ) -> RefCell<Vec<StackEntry>> {
    local_variables.append(arguments);
    RefCell::new(local_variables)
  }

  pub fn get_return_type(&self) -> &Vec<ValueTypes> {
    &self.function_instance.get_return_type()
  }

  pub fn get_return_count(&self) -> u32 {
    self.function_instance.get_return_count()
  }

  pub fn get_start_of_label(&self) -> u32 {
    self.ptr.get().sub(1)
  }

  pub fn peek(&self) -> Option<&u8> {
    let ptr = self.ptr.get();
    self.function_instance.get(ptr as usize)
  }

  pub fn pop_ref(&self) -> Option<&u8> {
    let head = self.peek();
    let ptr = self.ptr.get();
    self.ptr.set(ptr + 1);
    head
  }

  pub fn pop_runtime_type(&self) -> Option<ValueTypes> {
    match self.pop_ref() {
      Some(byte) => Some(ValueTypes::from(Some(*byte))),
      None => None,
    }
  }

  pub fn is_next_empty(&self) -> bool {
    match self.peek() {
      None => true,
      _ => false,
    }
  }

  pub fn jump_to(&self, ptr_of_label: u32) {
    self.ptr.replace(ptr_of_label);
  }

  pub fn jump_to_last(&self) {
    let last_ptr = self.last_ptr;
    self.jump_to(last_ptr);
  }

  pub fn get_table_address(&self) -> Indice {
    From::from(0u32)
  }
}

impl fmt::Debug for Frame {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_struct("Frame")
      .field(
        "type",
        &format!("{:?}", self.function_instance.get_function_type()),
      )
      .field("ptr", &self.ptr)
      .field("return_ptr", &self.return_ptr)
      .finish()
  }
}
