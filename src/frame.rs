use function::FunctionType;
use inst::Inst;
use std::fmt;
use store::Store;
use trap::Result;
use value::Values;
use value_type::ValueTypes;

#[derive(PartialEq, Clone)]
pub struct Frame {
  locals: &'a Vec<Values>,
  expressions: &'a Vec<Inst>,
  pub return_ptr: usize,
  // FIXME: May not need to store tables here, use instead of Store.
  pub table_addresses: Vec<u32>,
  pub own_type: FunctionType,
  pub ptr: u32,
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
      ptr: 0,
    })
  }

  pub fn is_completed(&self) -> bool {
    self.ptr >= self.last_ptr
  }

  pub fn is_fresh(&self) -> bool {
    self.ptr == 0
  }

  pub fn get_locals(&self) -> Vec<Values> {
    self.locals.to_owned()
  }

  pub fn peek(&self) -> Option<&Inst> {
    self.expressions.get(self.ptr as usize)
  }

  pub fn pop(&mut self) -> Option<Inst> {
    let head = self.expressions.get(self.ptr as usize).map(|x| x.clone());
    self.ptr += 1;
    head
  }

  pub fn pop_ref(&mut self) -> Option<&Inst> {
    let head = self.expressions.get(self.ptr as usize);
    self.ptr += 1;
    head
  }

  pub fn pop_runtime_type(&mut self) -> Option<ValueTypes> {
    match self.pop()? {
      Inst::RuntimeValue(ty) => Some(ty),
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

  pub fn jump_to(&mut self, ptr_of_label: u32) {
    self.ptr = ptr_of_label;
  }

  pub fn jump_to_last(&mut self) {
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
      self.own_type, locals, self.ptr, self.return_ptr, self.table_addresses
    )
  }
}
