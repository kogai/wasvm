use function::FunctionType;
use inst::{Inst, Instructions};
use std::fmt;
use std::rc::Rc;
use trap::{Result, Trap};
use value::Values;

#[derive(PartialEq, Clone)]
pub struct Frame {
  pub locals: Vec<Values>,
  pub expressions: Vec<Inst>,
  pub function_idx: usize,
  pub return_ptr: usize,
  pub table_addresses: Vec<u32>,
  pub types: Vec<Result<FunctionType>>,
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
      "[{}] locals:({}) return:{} table{:?}",
      self.function_idx, locals, self.return_ptr, self.table_addresses
    )
  }
}

#[derive(PartialEq)]
pub enum StackEntry {
  Empty,
  Value(Values),
  Label(Instructions),
  Frame(Frame),
}

impl fmt::Debug for StackEntry {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use StackEntry::*;
    let label = match self {
      Empty => "_".to_owned(),
      Value(v) => format!("{:?}", v),
      Label(v) => format!("Label({:?})", v),
      Frame(v) => format!("Frame({:?})", v),
    };
    write!(f, "{}", label)
  }
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
  fn is_empty(&self) -> bool {
    match self {
      StackEntry::Empty => true,
      _ => false,
    }
  }
}

pub struct Stack {
  stack_size: usize,
  entries: Vec<Rc<StackEntry>>,
  pub stack_ptr: usize,
  pub frame_ptr: Vec<usize>,
}

impl Stack {
  pub fn new(stack_size: usize) -> Self {
    let entries = vec![StackEntry::new_empty(); stack_size];
    Stack {
      stack_size,
      entries,
      stack_ptr: 0,
      frame_ptr: vec![],
    }
  }

  pub fn is_empty(&self) -> bool {
    self.stack_ptr == 0
  }

  pub fn get(&self, ptr: usize) -> Option<Rc<StackEntry>> {
    self.entries.get(ptr).map(|rc| rc.clone())
  }

  pub fn set(&mut self, ptr: usize, entry: Rc<StackEntry>) -> Result<()> {
    if ptr >= self.stack_size {
      return Err(Trap::StackOverflow);
    }
    self.entries[ptr] = entry;
    Ok(())
  }

  pub fn push(&mut self, entry: Rc<StackEntry>) -> Result<()> {
    if self.stack_ptr >= self.stack_size {
      return Err(Trap::StackOverflow);
    }
    self.entries[self.stack_ptr] = entry;
    self.stack_ptr += 1;
    Ok(())
  }

  pub fn pop(&mut self) -> Result<Rc<StackEntry>> {
    if self.stack_ptr <= 0 {
      return Err(Trap::StackOverflow);
    }
    self.stack_ptr -= 1;
    match self.entries.get(self.stack_ptr) {
      Some(entry) => Ok(entry.clone()),
      None => Err(Trap::Unknown),
    }
  }

  pub fn pop_value(&mut self) -> Result<Values> {
    let value = self.pop()?;
    match *value {
      StackEntry::Value(ref v) => Ok(v.to_owned()),
      _ => {
        self.push(value.clone())?;
        Err(Trap::Notfound)
      }
    }
  }
  pub fn pop_value_ext(&mut self) -> Values {
    self
      .pop_value()
      .expect("Expect to pop up value, but got None")
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
}

impl fmt::Debug for Stack {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let entries = self
      .entries
      .iter()
      .filter(|x| !x.is_empty())
      .map(|x| format!("{:?}", x))
      .collect::<Vec<String>>()
      .join(", ");

    write!(
      f,
      "{}, frame={:?}, stack_size={}, stack_ptr={}",
      format!("[{}]", entries),
      self.frame_ptr,
      self.stack_size,
      self.stack_ptr,
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn stack_push() {
    let mut stack = Stack::new(4);
    let value = StackEntry::new_value(Values::I32(1));
    stack.push(value).unwrap();
    assert_eq!(*stack.pop().unwrap(), StackEntry::Value(Values::I32(1)));
  }
  #[test]
  fn stack_set() {
    let mut stack = Stack::new(4);
    let value = StackEntry::new_value(Values::I32(2));
    stack.set(2, value).unwrap();
    assert_eq!(*stack.get(2).unwrap(), StackEntry::Value(Values::I32(2)));
  }
  #[test]
  fn stack_print() {
    let mut stack = Stack::new(8);
    for i in 0..3 {
      stack.push(StackEntry::new_value(Values::I32(i))).unwrap();
    }
    assert_eq!(
      format!("{:?}", stack),
      "[Value(I32(0)), Value(I32(1)), Value(I32(2))], frame=[], stack_size=8, stack_ptr=3"
        .to_owned()
    );
  }
}
