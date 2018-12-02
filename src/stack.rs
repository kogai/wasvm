use function::FunctionType;
use inst::Instructions;
use std::rc::Rc;
use trap::Result;
use value::Values;

#[derive(Debug, PartialEq, Clone)]
pub struct Frame {
  pub locals: Vec<Values>,
  pub function_idx: usize,
  pub return_ptr: usize,
  pub table_addresses: Vec<u32>,
  pub types: Vec<Result<FunctionType>>,
}

#[derive(Debug, PartialEq)]
pub enum StackEntry {
  Empty,
  Value(Values),
  Label(Instructions),
  Frame(Frame),
}

impl StackEntry {
  pub fn new_empty() -> Rc<Self> {
    Rc::new(StackEntry::Empty)
  }
  pub fn new_value(value: Values) -> Rc<Self> {
    Rc::new(StackEntry::Value(value))
  }
  pub fn new_label(label: Instructions) -> Rc<Self> {
    Rc::new(StackEntry::Label(label))
  }
  pub fn new_fram(frame: Frame) -> Rc<Self> {
    Rc::new(StackEntry::Frame(frame))
  }
}

#[derive(Debug)]
pub struct Stack {
  entries: Vec<Rc<StackEntry>>,
  pub stack_ptr: usize, // Start from 1
  pub frame_ptr: Vec<usize>,
  pub is_empty: bool,
}

impl Stack {
  pub fn new(stack_size: usize) -> Self {
    let entries = vec![StackEntry::new_empty(); stack_size];
    Stack {
      entries,
      stack_ptr: 0,
      frame_ptr: vec![],
      is_empty: false,
    }
  }

  pub fn increase(&mut self, count: usize) {
    self.stack_ptr += count;
  }

  pub fn get_frame_ptr(&mut self) -> usize {
    match self.frame_ptr.last() {
      Some(p) => *p,
      None => unreachable!("Frame pointer not found."),
    }
  }

  pub fn update_frame_ptr(&mut self) {
    match self.frame_ptr.pop() {
      Some(p) => {
        self.stack_ptr = p;
      }
      None => unreachable!("Frame pointer not found."),
    }
  }

  pub fn get(&self, ptr: usize) -> Option<Rc<StackEntry>> {
    self.entries.get(ptr).map(|rc| rc.clone())
  }

  pub fn set(&mut self, ptr: usize, entry: Rc<StackEntry>) {
    self.entries[ptr] = entry.clone();
  }

  pub fn push(&mut self, entry: Rc<StackEntry>) {
    self.entries[self.stack_ptr] = entry.clone();
    self.stack_ptr += 1;
  }

  pub fn pop(&mut self) -> Option<Rc<StackEntry>> {
    if self.stack_ptr == 0 {
      self.is_empty = true;
      None
    } else {
      self.stack_ptr -= 1;
      self.entries.get(self.stack_ptr).map(|rc| rc.clone())
    }
  }

  pub fn pop_value(&mut self) -> Option<Values> {
    let value = self.pop()?;
    match *value {
      StackEntry::Value(ref v) => Some(v.to_owned()),
      _ => {
        self.push(value.clone());
        None
      }
    }
  }
  pub fn pop_value_ext(&mut self) -> Values {
    self
      .pop_value()
      .expect("Expect to pop up value, but got None")
  }
}
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn stack_ptr() {
    let mut stack = Stack::new(4);
    stack.push(StackEntry::new_value(Values::I32(1)));
    stack.set(2, StackEntry::new_value(Values::I32(2)));
    assert_eq!(*stack.pop().unwrap(), StackEntry::Value(Values::I32(1)));
    assert_eq!(*stack.get(2).unwrap(), StackEntry::Value(Values::I32(2)));
  }
}
