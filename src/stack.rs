use frame::Frame;
use std::fmt;
use std::rc::Rc;
use store::Store;
use trap::{Result, Trap};
use value::Values;
use value_type::ValueTypes;

#[derive(PartialEq, Debug, Clone)]
pub enum LabelKind {
  If,
  LoopEnd,
  LoopContinuation,
  Block,
  // Nop,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Label {
  pub source_instruction: LabelKind,
  return_type: ValueTypes,
  pub continuation: u32,
}

#[derive(PartialEq, Clone)]
pub enum StackEntry {
  Empty,
  Pointer(usize),
  Value(Values),
  Label(Label),
  Frame(Frame),
}

impl fmt::Debug for StackEntry {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use self::StackEntry::*;
    let label = match self {
      Empty => "_".to_owned(),
      Pointer(v) => format!("P(*{:?})", v),
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
  pub fn new_empty() -> Rc<Self> {
    Rc::new(StackEntry::Empty)
  }
  pub fn new_value(value: Values) -> Rc<Self> {
    Rc::new(StackEntry::Value(value))
  }
  pub fn new_label(
    continuation: u32,
    return_type: ValueTypes,
    source_instruction: LabelKind,
  ) -> Rc<Self> {
    Rc::new(StackEntry::Label(Label {
      continuation,
      return_type,
      source_instruction: source_instruction,
    }))
  }

  pub fn new_frame(frame: Frame) -> Rc<Self> {
    Rc::new(StackEntry::Frame(frame))
  }

  pub fn new_pointer(ptr: usize) -> Rc<Self> {
    Rc::new(StackEntry::Pointer(ptr))
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
      match *value {
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
/// | ..            |
/// | Empty         | < Stack pointer
/// | Locals..      |
/// | Local 1       |
/// | Local 0       |
/// | Args..        |
/// | Args  1       |
/// | Args  0       | Indices are starts by zero.
/// | ReturnPointer |
/// | ...           | < Frame pointer
pub struct Stack {
  stack_size: usize,
  entries: Vec<Rc<StackEntry>>,
  pushed_frame: usize,
  pub stack_ptr: usize,
  pub frame_ptr: usize,
}

impl Stack {
  pub fn new(stack_size: usize) -> Self {
    let entries = vec![StackEntry::new_empty(); stack_size];
    Stack {
      stack_size,
      entries,
      pushed_frame: 0,
      stack_ptr: 0,
      frame_ptr: 0,
    }
  }

  pub fn is_empty(&self) -> bool {
    self.stack_ptr == 0
  }

  pub fn get(&self, ptr: usize) -> Option<Rc<StackEntry>> {
    self.entries.get(ptr).map(|x| x.clone())
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

  pub fn push_entries(&mut self, entries: &mut Vec<Rc<StackEntry>>) -> Result<()> {
    while let Some(entry) = entries.pop() {
      self.push(entry)?;
    }
    Ok(())
  }

  pub fn push_frame(
    &mut self,
    store: &mut Store,
    function_idx: usize,
    arguments: &mut Vec<Values>,
  ) -> Result<()> {
    let frame = Frame::new(store, self.stack_ptr, function_idx, arguments)?;
    self.push(StackEntry::new_frame(frame))?;
    self.pushed_frame += 1;
    Ok(())
  }

  pub fn decrease_pushed_frame(&mut self) {
    self.pushed_frame -= 1;
  }

  pub fn is_frame_ramained(&self) -> bool {
    self.pushed_frame > 0
  }

  pub fn peek(&self) -> Option<Rc<StackEntry>> {
    if self.stack_ptr >= self.stack_size {
      return None;
    }
    if self.stack_ptr <= 0 {
      return None;
    }
    self.entries.get(self.stack_ptr - 1).map(|x| x.clone())
  }

  pub fn pop(&mut self) -> Result<Rc<StackEntry>> {
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
  impl_pop!(pop_frame, pop_frame_ext, StackEntry::Frame, Frame, "Frame");

  pub fn pop_until(&mut self, kind: &StackEntryKind) -> Result<Vec<Rc<StackEntry>>> {
    let mut entry_buffer = vec![];
    while !self.peek().map_or(true, |entry| entry.is_same_kind(kind)) {
      entry_buffer.push(self.pop()?);
    }
    Ok(entry_buffer)
  }

  pub fn jump_to_label(&mut self, depth_of_label: u32) -> Result<u32> /* point to continue */ {
    let mut buf_values: Vec<Rc<StackEntry>> = vec![];
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
        ..
      }) => continuation,
      Some(Label {
        return_type: _,
        continuation,
        ..
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
    self.frame_ptr
  }

  pub fn update_frame_ptr(&mut self) {
    let frame_ptr = self
      .get(self.frame_ptr)
      .expect("Expected Frame pointer, got None");
    match *frame_ptr {
      StackEntry::Pointer(ref p) => {
        self.stack_ptr = self.frame_ptr;
        self.frame_ptr = *p;
      }
      ref x => unreachable!("Expected Frame pointer, got {:?}", x),
    }
  }
}

impl fmt::Debug for Stack {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let (entries, _) = self.entries.split_at(self.stack_ptr);
    let entries = entries
      .iter()
      .enumerate()
      .map(|(i, entry)| match i + 1 {
        x if x == self.frame_ptr => format!("F-> {:?}", entry),
        x if x == self.stack_ptr => format!("S-> {:?}", entry),
        _ => format!("    {:?}", entry),
      })
      .rev();
    f.debug_list().entries(entries).finish()
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
}
