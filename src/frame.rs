use function::FunctionType;
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
  locals: Vec<Values>,
  expressions: Rc<Vec<Inst>>,
  pub return_ptr: usize,
  // FIXME: May not need to store tables here, use instead of Store.
  pub table_addresses: Vec<u32>,
  pub own_type: FunctionType,
  ptr: RefCell<u32>,
  last_ptr: u32,
}

impl Frame {
  pub fn new(
    store: &mut Store,
    return_ptr: usize,
    function_idx: usize,
    locals: &mut Vec<Values>,
  ) -> Result<Self> {
    let function_instance = store.get_function_instance(function_idx)?;
    let own_type = function_instance.get_function_type()?;
    let (expressions, local_types) = function_instance.call();
    for local in local_types {
      locals.push(Values::from(local));
    }
    Ok(Frame {
      locals: locals.to_owned(),
      last_ptr: expressions.len() as u32,
      expressions,
      return_ptr,
      table_addresses: vec![0],
      own_type,
      ptr: RefCell::new(0),
    })
  }

  pub fn is_completed(&self) -> bool {
    self.ptr.borrow().ge(&self.last_ptr)
  }

  pub fn is_fresh(&self) -> bool {
    self.ptr.borrow().eq(&0)
  }

  // TODO: Consider to necessity of ownership.
  pub fn get_locals(&self) -> Vec<Values> {
    self.locals.to_owned()
  }

  pub fn get_start_of_label(&self) -> u32 {
    self.ptr.borrow().sub(1)
  }

  pub fn peek(&self) -> Option<&Inst> {
    let ptr = self.ptr.borrow();
    self.expressions.get(*ptr as usize)
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

  pub fn jump_to(&self, ptr_of_label: u32) {
    self.ptr.replace(ptr_of_label);
  }

  pub fn jump_to_last(&self) {
    let last_ptr = self.last_ptr;
    self.jump_to(last_ptr);
  }

  pub fn get_table_address(&self) -> u32 {
    *self
      .table_addresses
      .get(0)
      .expect("Table address [0] not found")
  }

  pub fn increment_return_ptr(&mut self) {
    self.return_ptr += 1;
  }
}

impl fmt::Debug for Frame {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    // NOTE: Omit to present expressions and types would be worth :thinking: .
    let locals = self
      .locals
      .iter()
      .map(|x| format!("{:?}", x))
      .collect::<Vec<String>>()
      .join(", ");
    write!(
      f,
      "{:?} locals: ({}) ptr: {} return:{} table{:?}",
      self.own_type,
      locals,
      self.ptr.borrow(),
      self.return_ptr,
      self.table_addresses
    )
  }
}
