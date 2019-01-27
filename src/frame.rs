use alloc::vec::Vec;
use core::cell::{RefCell, RefMut};
use core::fmt;
use core::ops::{AddAssign, Sub};
use function::FunctionInstance;
use isa::Indice;
use stack::StackEntry;
use trap::Result;
use value_type::ValueTypes;

#[derive(PartialEq)]
pub struct Frame {
  // FIXME: No need to hold local_variables in frame.
  local_variables: RefCell<Vec<StackEntry>>,
  pub(crate) function_instance: FunctionInstance,
  ptr: RefCell<u32>,
  pub last_ptr: u32, // FIXME: Use Indice type for indices of instructions.
  pub return_ptr: usize,
  pub prev_return_ptr: usize,
}

impl Frame {
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
      ptr: RefCell::new(0),
    }
  }

  pub fn is_completed(&self) -> bool {
    self.ptr.borrow().ge(&self.last_ptr)
  }

  pub fn is_fresh(&self) -> bool {
    self.ptr.borrow().eq(&0)
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
    self.ptr.borrow().sub(1)
  }

  pub fn peek(&self) -> Option<&u8> {
    let ptr = self.ptr.borrow();
    self.function_instance.get(*ptr as usize)
  }

  pub fn pop_ref(&self) -> Option<&u8> {
    let head = self.peek();
    self.ptr.borrow_mut().add_assign(1);
    head
  }

  pub fn pop_runtime_type(&self) -> Option<ValueTypes> {
    match self.pop_ref() {
      Some(byte) => Some(ValueTypes::from(Some(*byte))),
      None => None,
    }
  }

  pub fn pop_raw_u32(&self) -> Result<u32> {
    let mut buf = [0; 4];
    for i in 0..buf.len() {
      buf[i] = *self.pop_ref()?;
    }
    let idx = unsafe { core::mem::transmute::<_, u32>(buf) };
    Ok(idx)
  }

  pub fn pop_raw_u64(&self) -> Result<u64> {
    let mut buf = [0; 8];
    for i in 0..buf.len() {
      buf[i] = *self.pop_ref()?;
    }
    let idx = unsafe { core::mem::transmute::<_, u64>(buf) };
    Ok(idx)
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
