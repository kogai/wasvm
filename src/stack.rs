use function::FunctionType;
use inst::Inst;
use std::fmt;
use trap::{Result, Trap};
use value::Values;
use value_type::ValueTypes;

#[derive(PartialEq, Debug, Clone)]
pub struct Label {
  return_type: ValueTypes,
  continuation: u32,
}

#[derive(PartialEq, Clone)]
pub struct Frame {
  pub locals: Vec<Values>,
  pub expressions: Vec<Inst>,
  pub function_idx: usize,
  pub return_ptr: usize,
  pub table_addresses: Vec<u32>,
  pub own_type: Option<FunctionType>,
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

#[derive(PartialEq, Clone)]
pub enum StackEntry {
  Empty,
  Value(Values),
  Label(Label),
  Frame(Frame),
}

impl fmt::Debug for StackEntry {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use self::StackEntry::*;
    let label = match self {
      Empty => "_".to_owned(),
      Value(v) => format!("{:?}", v),
      Label(v) => format!("{:?}", v),
      Frame(v) => format!("Frame({:?})", v),
    };
    write!(f, "{}", label)
  }
}

#[allow(dead_code)]
pub enum StackEntryKind {
  Empty,
  Value,
  Label,
  Frame,
}

// pub const STACK_ENTRY_KIND_EMPTY: StackEntryKind = StackEntryKind::Empty;
// pub const STACK_ENTRY_KIND_VALUE: StackEntryKind = StackEntryKind::Value;
pub const STACK_ENTRY_KIND_LABEL: StackEntryKind = StackEntryKind::Label;
pub const STACK_ENTRY_KIND_FRAME: StackEntryKind = StackEntryKind::Frame;

impl StackEntry {
  pub fn new_empty() -> Self {
    StackEntry::Empty
  }
  pub fn new_value(value: Values) -> Self {
    StackEntry::Value(value)
  }
  pub fn new_label(continuation: u32, return_type: ValueTypes) -> Self {
    StackEntry::Label(Label {
      continuation,
      return_type,
    })
  }
  pub fn new_fram(frame: Frame) -> Self {
    StackEntry::Frame(frame)
  }

  fn is_same_kind(&self, other: &StackEntryKind) -> bool {
    use self::StackEntry::*;
    match (self, other) {
      (Empty, StackEntryKind::Empty)
      | (Value(_), StackEntryKind::Value)
      | (Label(_), StackEntryKind::Label)
      | (Frame(_), StackEntryKind::Frame) => true,
      _ => false,
    }
  }
}

macro_rules! impl_pop {
  ($name: ident, $name_ext: ident, $path: path, $ret: ty, $kind: expr) => {
    pub fn $name(&mut self) -> Result<$ret> {
      let value = self.pop()?;
      match value {
        $path(ref v) => Ok(v.to_owned()),
        _ => {
          self.push(value.to_owned())?;
          Err(Trap::Notfound)
        }
      }
    }

    pub fn $name_ext(&mut self) -> $ret {
      self
        .$name()
        .expect(&format!("Expect to pop up {}, but got None", $kind))
    }
  };
}

macro_rules! impl_pop_value_ext {
  ($name: ident, $path: path, $ret: ty) => {
    pub fn $name(&mut self) -> $ret {
      match self.pop_value_ext() {
        $path(n) => n,
        _ => unreachable!(),
      }
    }
  };
}

/// Layout of Stack
///
/// | ..      |
/// | Empty   | < Stack pointer
/// | Local.. |
/// | Local 2 |
/// | Local 1 |
/// | Args .. |
/// | Args  2 |
/// | Args  1 |
pub struct Stack {
  stack_size: usize,
  entries: Vec<StackEntry>,
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

  pub fn get(&self, ptr: usize) -> Option<StackEntry> {
    self.entries.get(ptr).map(|rc| rc.to_owned())
  }

  pub fn set(&mut self, ptr: usize, entry: StackEntry) -> Result<()> {
    if ptr >= self.stack_size {
      return Err(Trap::StackOverflow);
    }
    self.entries[ptr] = entry;
    Ok(())
  }

  pub fn push(&mut self, entry: StackEntry) -> Result<()> {
    if self.stack_ptr >= self.stack_size {
      return Err(Trap::StackOverflow);
    }
    self.entries[self.stack_ptr] = entry;
    self.stack_ptr += 1;
    Ok(())
  }

  pub fn push_entries(&mut self, entries: &mut Vec<StackEntry>) -> Result<()> {
    while let Some(entry) = entries.pop() {
      self.push(entry)?;
    }
    Ok(())
  }

  pub fn peek(&self) -> Option<&StackEntry> {
    if self.stack_ptr >= self.stack_size {
      return None;
    }
    if self.stack_ptr <= 0 {
      return None;
    }
    self.entries.get(self.stack_ptr - 1)
  }

  pub fn pop(&mut self) -> Result<StackEntry> {
    if self.stack_ptr <= 0 {
      return Err(Trap::StackUnderflow);
    }
    self.stack_ptr -= 1;
    match self.entries.get(self.stack_ptr) {
      Some(entry) => Ok(entry.clone()),
      None => Err(Trap::Unknown),
    }
  }

  impl_pop!(pop_value, pop_value_ext, StackEntry::Value, Values, "Value");
  impl_pop!(pop_label, pop_label_ext, StackEntry::Label, Label, "Label");

  pub fn pop_until(&mut self, kind: &StackEntryKind) -> Result<Vec<StackEntry>> {
    let mut entry_buffer = vec![];
    while !self.peek().map_or(true, |entry| entry.is_same_kind(kind)) {
      entry_buffer.push(self.pop()?);
    }
    Ok(entry_buffer)
  }

  pub fn jump_to_label(&mut self, depth_of_label: u32) -> Result<u32> /* point to continue */ {
    let mut buf_values: Vec<StackEntry> = vec![];
    let mut label = None;
    for _ in 0..(depth_of_label + 1) {
      let mut bufs = self.pop_until(&StackEntryKind::Label)?;
      buf_values.append(&mut bufs);
      label = Some(self.pop_label_ext());
    }
    let continuation = match label {
      Some(Label {
        return_type: ValueTypes::Empty,
        continuation,
      }) => continuation,
      Some(Label {
        return_type: _,
        continuation,
      }) => {
        // FIXME: Prefer to pop and push with count of return_types.
        let return_val = buf_values
          .first()
          .expect("At least one return value should exists.")
          .to_owned();
        self.push(return_val)?;
        continuation
      }
      x => unreachable!("At least one label should exists.\n{:?}", x),
    };
    Ok(continuation)
  }

  impl_pop_value_ext!(pop_value_ext_i32, Values::I32, i32);
  // NOTE: May not needed?
  // impl_pop_value_ext!(pop_value_ext_i64, Values::I64, i64);
  // impl_pop_value_ext!(pop_value_ext_f32, Values::F32, f32);
  // impl_pop_value_ext!(pop_value_ext_f64, Values::F64, f64);

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
    assert_eq!(stack.pop().unwrap(), StackEntry::Value(Values::I32(1)));
  }
  #[test]
  fn stack_set() {
    let mut stack = Stack::new(4);
    let value = StackEntry::new_value(Values::I32(2));
    stack.set(2, value).unwrap();
    assert_eq!(stack.get(2).unwrap(), StackEntry::Value(Values::I32(2)));
  }
  #[test]
  fn stack_print() {
    let mut stack = Stack::new(8);
    for i in 0..3 {
      stack.push(StackEntry::new_value(Values::I32(i))).unwrap();
    }
    assert_eq!(
      format!("{:?}", stack),
      "[i32:0, i32:1, i32:2, _, _, _, _, _], frame=[], stack_size=8, stack_ptr=3".to_owned()
    );
  }
}
