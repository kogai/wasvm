#[cfg(not(test))]
use alloc::prelude::*;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::{Cell, RefCell};
use core::fmt;
use error::{Result, Trap, WasmError};
use frame::Frame;
use indice::Indice;
use label::{Label, LabelKind};
use value::Values;
use value_type::ValueTypes;

#[derive(PartialEq)]
enum StackEntryImpl {
  Empty,
  Value(Values),
  Label(Label),
}

#[derive(PartialEq, Clone)]
pub struct StackEntry(Rc<StackEntryImpl>);

impl fmt::Debug for StackEntry {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use self::StackEntryImpl::*;
    let label = match *self.0 {
      Empty => "_".to_owned(),
      Value(ref v) => format!("{:?}", v),
      Label(ref v) => format!("{:?}", v),
    };
    write!(f, "{}", label)
  }
}

impl StackEntry {
  fn new(entry: StackEntryImpl) -> Self {
    StackEntry(Rc::new(entry))
  }

  pub fn new_empty() -> Self {
    StackEntry::new(StackEntryImpl::Empty)
  }

  pub fn new_value(value: Values) -> Self {
    StackEntry::new(StackEntryImpl::Value(value))
  }

  pub fn new_label(
    continuation: u32,
    return_type: ValueTypes,
    source_instruction: LabelKind,
  ) -> Self {
    StackEntry::new(StackEntryImpl::Label(Label {
      continuation,
      return_type,
      source_instruction,
    }))
  }

  fn is_label(&self) -> bool {
    use self::StackEntryImpl::*;
    match *self.0 {
      Label(_) => true,
      _ => false,
    }
  }
}

macro_rules! impl_pop {
  ($name: ident, $name_ext: ident, $path: path, $ret: ty, $error_decription: expr) => {
    pub fn $name(&self) -> Result<$ret> {
      let value = self.pop()?;
      match *value.0 {
        $path(ref v) => Ok(v.to_owned()),
        _ => {
          self.push(value.to_owned())?;
          Err(WasmError::Trap(Trap::Notfound))
        }
      }
    }

    pub fn $name_ext(&self) -> $ret {
      self
        .$name()
        .expect($error_decription)
    }
  };
}

macro_rules! impl_pop_value_ext {
  ($name: ident, $path: path, $ret: ty) => {
    pub fn $name(&self) -> $ret {
      match self.pop_value_ext() {
        $path(n) => n,
        _ => unreachable!(),
      }
    }
  };
}

/// Layout of Operand Stack
///
/// +---------------+
/// | ..            |
/// +---------------+
/// | Empty*        | <- Stack pointer
/// +---------------+
/// | Locals*       |
/// +---------------+
/// | Local 1       |
/// +---------------+
/// | Local 0       |
/// +---------------+
/// | Args*         |
/// +---------------+
/// | Args  1       |
/// +---------------+
/// | Args  0       | Indices are starts by zero.
/// +---------------+
/// | ...           | <- Frame pointer
/// +---------------+
pub struct Stack {
  pub(crate) stack_size: usize,
  operand_stack: RefCell<Vec<StackEntry>>,
  call_stack: RefCell<Vec<Frame>>,
  pub(crate) stack_ptr: Cell<usize>,
  pub(crate) frame_ptr: Cell<usize>,
}

impl Stack {
  pub fn new(stack_size: usize) -> Self {
    let operand_stack = RefCell::new(vec![StackEntry::new_empty(); stack_size]);
    let call_stack = RefCell::new(Vec::with_capacity(stack_size));
    Stack {
      stack_size,
      operand_stack,
      call_stack,
      stack_ptr: Cell::new(0),
      frame_ptr: Cell::new(0),
    }
  }

  pub(crate) fn stack_ptr(&self) -> usize {
    self.stack_ptr.get()
  }

  pub(crate) fn frame_ptr(&self) -> usize {
    self.frame_ptr.get()
  }

  pub fn get(&self, ptr: usize) -> Option<StackEntry> {
    self.operand_stack.borrow().get(ptr).cloned()
  }

  pub fn set(&self, ptr: usize, entry: StackEntry) -> Result<()> {
    if ptr >= self.stack_size {
      return Err(WasmError::Trap(Trap::StackOverflow));
    }
    self.operand_stack.borrow_mut()[ptr] = entry;
    Ok(())
  }

  pub fn push(&self, entry: StackEntry) -> Result<()> {
    if self.stack_ptr() >= self.stack_size {
      return Err(WasmError::Trap(Trap::StackOverflow));
    }
    self.operand_stack.borrow_mut()[self.stack_ptr()] = entry;
    self.stack_ptr.set(self.stack_ptr() + 1);
    Ok(())
  }

  /// From entries [2,1,0,..];
  /// To stack below.
  /// +---------+
  /// | Val*    |
  /// +---------+
  /// | Val2    |
  /// +---------+
  /// | Val1    |
  /// +---------+
  /// | Val0    |
  /// +---------+
  pub fn push_entries(&self, entries: &mut Vec<StackEntry>) -> Result<()> {
    let len = entries.len();
    let stack_ptr = self.stack_ptr();
    let stack_ptr_end = stack_ptr + len;
    if stack_ptr_end >= self.stack_size {
      Err(WasmError::Trap(Trap::StackOverflow))
    } else {
      entries.reverse();
      entries.swap_with_slice(&mut self.operand_stack.borrow_mut()[stack_ptr..stack_ptr_end]);
      self.stack_ptr.set(stack_ptr_end);
      Ok(())
    }
  }

  pub fn push_frame(&self, frame: Frame) -> Result<()> {
    self.call_stack.borrow_mut().push(frame);
    Ok(())
  }

  pub fn push_back_frame(&self, frame: Frame) {
    let mut calls = self.call_stack.borrow_mut();
    let len = calls.len();
    calls.push(frame);
    calls.swap(len, len - 1);
  }

  pub fn pop_frame(&self) -> Option<Frame> {
    let mut calls = self.call_stack.borrow_mut();
    calls.pop()
  }

  pub fn call_stack_is_empty(&self) -> bool {
    let calls = self.call_stack.borrow();
    calls.is_empty()
  }

  pub fn peek(&self) -> Option<StackEntry> {
    if self.stack_ptr() >= self.stack_size {
      return None;
    }
    if self.stack_ptr() == 0 {
      return None;
    }
    self
      .operand_stack
      .borrow_mut()
      .get(self.stack_ptr() - 1)
      .cloned()
  }

  pub fn pop(&self) -> Result<StackEntry> {
    if self.stack_ptr() == 0 {
      return Err(WasmError::Trap(Trap::StackUnderflow));
    }
    self.stack_ptr.set(self.stack_ptr() - 1);
    match self.operand_stack.borrow_mut().get(self.stack_ptr()) {
      Some(entry) => Ok(entry.clone()),
      None => Err(WasmError::Trap(Trap::Unknown)),
    }
  }

  impl_pop!(
    pop_value,
    pop_value_ext,
    StackEntryImpl::Value,
    Values,
    "Expect to pop up Value, but got None"
  );
  impl_pop!(
    pop_label,
    pop_label_ext,
    StackEntryImpl::Label,
    Label,
    "Expect to pop up Label, but got None"
  );

  pub fn pop_until_label(&self) -> Result<Vec<StackEntry>> {
    let mut entry_buffer = vec![];
    while !self.peek().map_or(true, |entry| entry.is_label()) {
      entry_buffer.push(self.pop()?);
    }
    Ok(entry_buffer)
  }

  pub fn jump_to_label(&self, depth_of_label: &Indice) -> Result<u32> /* point to continue */ {
    let mut buf_values = vec![];
    let mut label = None;
    for _ in 0..=depth_of_label.to_u32() {
      let mut bufs = self.pop_until_label()?;
      buf_values.append(&mut bufs);
      label = Some(self.pop_label_ext());
    }
    let continuation = match label {
      Some(Label {
        return_type: ValueTypes::Unit,
        continuation,
        ..
      }) => continuation,
      Some(Label {
        continuation,
        source_instruction,
        ..
      }) => {
        match source_instruction {
          LabelKind::Loop => {
            self.push_entries(&mut buf_values)?;
          }
          _ => {
            // FIXME: Prefer to pop and push with count of return_types.
            let return_val = buf_values
              .first()
              .expect("At least one return value should exists.")
              .to_owned();
            self.push(return_val)?;
          }
        };
        continuation
      }
      x => unreachable!("At least one label should exists.\n{:?}", x),
    };
    Ok(continuation)
  }

  impl_pop_value_ext!(pop_value_ext_i32, Values::I32, i32);

  pub fn update_frame_ptr(&self, frame: &Frame) {
    self.stack_ptr.set(self.frame_ptr());
    self.frame_ptr.set(frame.prev_return_ptr);
  }
}

impl fmt::Debug for Stack {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let operands = self.operand_stack.borrow();
    let (entries, _) = operands.split_at(self.stack_ptr());
    let entries = entries
      .iter()
      .enumerate()
      .map(|(i, entry)| match i + 1 {
        x if x == self.frame_ptr() => format!("F-> {:?}", entry),
        x if x == self.stack_ptr() => format!("S-> {:?}", entry),
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
    let stack = Stack::new(4);
    let value = StackEntry::new_value(Values::I32(1));
    stack.push(value).unwrap();
    assert_eq!(stack.pop().unwrap(), StackEntry::new_value(Values::I32(1)));
  }
  #[test]
  fn stack_set() {
    let stack = Stack::new(4);
    let value = StackEntry::new_value(Values::I32(2));
    stack.set(2, value).unwrap();
    assert_eq!(stack.get(2).unwrap(), StackEntry::new_value(Values::I32(2)));
  }
}
