use function::FunctionInstance;
use inst::Inst;
use std::cell::RefCell;
use std::fmt;
use std::ops::{AddAssign, Sub};
use std::rc::Rc;
use store::Store;
use trap::Result;
use value::Values;
use value_type::ValueTypes;

#[derive(PartialEq, Clone)]
pub struct Frame {
  arguments: Vec<Values>,
  function_instance: Rc<FunctionInstance>,
  ptr: RefCell<u32>,
  last_ptr: u32,
  pub return_ptr: usize,
}

impl Frame {
  pub fn new(
    store: &mut Store,
    return_ptr: usize,
    function_idx: usize,
    arguments: Vec<Values>,
  ) -> Result<Self> {
    let function_instance = store.get_function_instance(function_idx)?;
    let last_ptr = function_instance.get_expressions_count() as u32;
    Ok(Frame {
      function_instance,
      arguments,
      last_ptr,
      return_ptr,
      ptr: RefCell::new(0),
    })
  }

  pub fn is_completed(&self) -> bool {
    self.ptr.borrow().ge(&self.last_ptr)
  }

  pub fn is_fresh(&self) -> bool {
    self.ptr.borrow().eq(&0)
  }

  pub fn get_locals(&self) -> Vec<Values> {
    let mut arguments = self.arguments.to_owned();
    for local in &self.function_instance.locals {
      arguments.push(Values::from(local));
    }
    arguments
  }

  pub fn get_return_count(&self) -> u32 {
    self
      .function_instance
      .get_function_type()
      .unwrap()
      .get_return_count()
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

  pub fn increment_return_ptr(&mut self) {
    self.return_ptr += 1;
  }
}

impl fmt::Debug for Frame {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    // NOTE: Omit to present expressions and types would be worth :thinking: .
    let locals = self
      .get_locals()
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
