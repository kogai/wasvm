use inst::Inst;
use std::rc::Rc;
use value::Values;

#[derive(Debug, PartialEq, Clone)]
pub struct Frame {
  pub locals: Vec<Values>,
  pub function_idx: usize,
  pub return_ptr: usize,
}

#[derive(Debug, PartialEq)]
pub enum StackEntry {
  Empty,
  Value(Values),
  Label(Vec<Inst>),
  Frame(Frame),
}

impl StackEntry {
  pub fn new_empty() -> Rc<Self> {
    Rc::new(StackEntry::Empty)
  }
  pub fn new_value(value: Values) -> Rc<Self> {
    Rc::new(StackEntry::Value(value))
  }
  pub fn new_label(label: Vec<Inst>) -> Rc<Self> {
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

  pub fn pop_value(&mut self) -> Values {
    let value = self.pop().expect("Expect to popp value but got None");
    match *value {
      StackEntry::Value(ref v) => v.to_owned(),
      ref x => unreachable!(format!("Expect to popp value but got {:?}", x).as_str()),
    }
  }
}
