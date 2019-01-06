use alloc::prelude::*;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::{RefCell, RefMut};
use core::fmt;
use core::ops::{AddAssign, Sub};
use function::FunctionInstance;
use inst::Inst;
use stack::StackEntry;
use value_type::ValueTypes;

#[derive(PartialEq)]
pub struct Frame {
  local_variables: RefCell<Vec<Rc<StackEntry>>>,
  function_instance: Rc<FunctionInstance>,
  ptr: RefCell<u32>,
  pub last_ptr: u32,
  pub return_ptr: usize,
}

impl Frame {
  pub fn new(
    return_ptr: usize,
    function_instance: Rc<FunctionInstance>,
    arguments: &mut Vec<Rc<StackEntry>>,
  ) -> Self {
    let last_ptr = function_instance.get_expressions_count() as u32;
    Frame {
      local_variables: Frame::derive_local_variables(
        arguments,
        function_instance.local_variables.clone(),
      ),
      function_instance,
      last_ptr,
      return_ptr,
      ptr: RefCell::new(0),
    }
  }

  pub fn is_completed(&self) -> bool {
    self.ptr.borrow().ge(&self.last_ptr)
  }

  pub fn is_fresh(&self) -> bool {
    self.ptr.borrow().eq(&0)
  }

  pub fn get_local_variables(&self) -> RefMut<Vec<Rc<StackEntry>>> {
    self.local_variables.borrow_mut()
  }

  // From: args[2,1]; locals[4,3]
  // To [4,3,2,1]
  fn derive_local_variables(
    arguments: &mut Vec<Rc<StackEntry>>,
    mut local_variables: Vec<Rc<StackEntry>>,
  ) -> RefCell<Vec<Rc<StackEntry>>> {
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

  pub fn peek(&self) -> Option<&Inst> {
    let ptr = self.ptr.borrow();
    self.function_instance.get(*ptr as usize)
  }

  pub fn pop_ref(&self) -> Option<&Inst> {
    let head = self.peek();
    self.ptr.borrow_mut().add_assign(1);
    head
  }

  pub fn pop_runtime_type(&self) -> Option<ValueTypes> {
    match self.pop_ref()? {
      Inst::RuntimeValue(ty) => Some(ty.to_owned()),
      _ => None,
    }
  }

  pub fn is_next_empty(&self) -> bool {
    match self.peek() {
      None => true,
      _ => false,
    }
  }

  pub fn is_next_end(&self) -> bool {
    match self.peek() {
      Some(Inst::End) | None => true,
      _ => false,
    }
  }

  pub fn is_next_else(&self) -> bool {
    match self.peek() {
      Some(Inst::Else) => true,
      _ => false,
    }
  }

  pub fn is_next_end_or_else(&self) -> bool {
    self.is_next_end() || self.is_next_else()
  }

  // TODO: Prefert to define as private function
  pub fn jump_to(&self, ptr_of_label: u32) {
    self.ptr.replace(ptr_of_label);
  }

  pub fn jump_to_last(&self) {
    let last_ptr = self.last_ptr;
    self.jump_to(last_ptr);
  }

  pub fn get_table_address(&self) -> u32 {
    0
  }
}

impl fmt::Debug for Frame {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    // NOTE: Omit to present expressions and types would be worth :thinking: .
    let locals = self
      .local_variables
      .borrow()
      .iter()
      .map(|x| format!("{:?}", x))
      .collect::<Vec<String>>()
      .join(", ");
    write!(
      f,
      "{:?} locals: ({}) ptr: {} return:{}",
      self.function_instance.get_function_type(),
      locals,
      self.ptr.borrow(),
      self.return_ptr,
    )
  }
}
