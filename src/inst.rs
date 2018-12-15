use function::FunctionType;
use std::convert::Into;
use std::fmt;
use trap::Result;
use value::Values;
use value_type::ValueTypes;

#[derive(Debug, PartialEq, Clone)]
pub enum Inst {
  Unreachable,
  Nop,
  Block(u32),
  Loop,
  If(u32, u32),
  Else,
  End,
  Br(u32),
  BrIf(u32),
  BrTable(Vec<u32>, u32),
  Return,
  Call(usize), // FIXME: Change to u32
  CallIndirect(u32),

  I32Const(i32),
  I64Const(i64),
  F32Const(f32),
  F64Const(f64),

  GetLocal(u32),
  SetLocal(u32),
  TeeLocal(u32),
  GetGlobal(u32),
  SetGlobal(u32),

  I32Load(u32, u32),
  I64Load(u32, u32),
  F32Load(u32, u32),
  F64Load(u32, u32),
  I32Load8Sign(u32, u32),
  I32Load8Unsign(u32, u32),
  I32Load16Sign(u32, u32),
  I32Load16Unsign(u32, u32),
  I64Load8Sign(u32, u32),
  I64Load8Unsign(u32, u32),
  I64Load16Sign(u32, u32),
  I64Load16Unsign(u32, u32),
  I64Load32Sign(u32, u32),
  I64Load32Unsign(u32, u32),
  I32Store(u32, u32),
  I64Store(u32, u32),
  F32Store(u32, u32),
  F64Store(u32, u32),
  I32Store8(u32, u32),
  I32Store16(u32, u32),
  I64Store8(u32, u32),
  I64Store16(u32, u32),
  I64Store32(u32, u32),
  MemorySize,
  MemoryGrow,

  I32CountLeadingZero,
  I32CountTrailingZero,
  I32CountNonZero,
  I32Add,
  I32Sub,
  I32Mul,
  I32DivSign,
  I32DivUnsign,
  I32RemSign,
  I32RemUnsign,
  I32And,
  I32Or,
  I32Xor,
  I32ShiftLeft,
  I32ShiftRIghtSign,
  I32ShiftRightUnsign,
  I32RotateLeft,
  I32RotateRight,

  I64CountLeadingZero,
  I64CountTrailingZero,
  I64CountNonZero,
  I64Add,
  I64Sub,
  I64Mul,
  I64DivSign,
  I64DivUnsign,
  I64RemSign,
  I64RemUnsign,
  I64And,
  I64Or,
  I64Xor,
  I64ShiftLeft,
  I64ShiftRightSign,
  I64ShiftRightUnsign,
  I64RotateLeft,
  I64RotateRight,

  I32EqualZero,
  Equal,
  NotEqual,
  LessThanSign,
  LessThanUnsign,
  I32GreaterThanSign,
  I32GreaterThanUnsign,
  I32LessEqualSign,
  I32LessEqualUnsign,
  I32GreaterEqualSign,
  I32GreaterEqualUnsign,

  I64EqualZero,
  I64Equal,
  I64NotEqual,
  I64LessThanSign,
  I64LessThanUnSign,
  I64GreaterThanSign,
  I64GreaterThanUnSign,
  I64LessEqualSign,
  I64LessEqualUnSign,
  I64GreaterEqualSign,
  I64GreaterEqualUnSign,

  F32Equal,
  F32NotEqual,
  F32LessThan,
  F32GreaterThan,
  F32LessEqual,
  F32GreaterEqual,
  F64Equal,
  F64NotEqual,
  F64LessThan,
  F64GreaterThan,
  F64LessEqual,
  F64GreaterEqual,

  F32Abs,
  F32Neg,
  F32Ceil,
  F32Floor,
  F32Trunc,
  F32Nearest,
  F32Sqrt,
  F32Add,
  F32Sub,
  F32Mul,
  F32Div,
  F32Min,
  F32Max,
  F32Copysign,

  F64Abs,
  F64Neg,
  F64Ceil,
  F64Floor,
  F64Trunc,
  F64Nearest,
  F64Sqrt,
  F64Add,
  F64Sub,
  F64Mul,
  F64Div,
  F64Min,
  F64Max,
  F64Copysign,

  Select,
  DropInst,
  I32WrapI64,

  I32TruncSignF32,
  I32TruncUnsignF32,
  I32TruncSignF64,
  I32TruncUnsignF64,
  I64ExtendSignI32,
  I64ExtendUnsignI32,
  I64TruncSignF32,
  I64TruncUnsignF32,
  I64TruncSignF64,
  I64TruncUnsignF64,
  F32ConvertSignI32,
  F32ConvertUnsignI32,
  F32ConvertSignI64,
  F32ConvertUnsignI64,
  F32DemoteF64,
  F64ConvertSignI32,
  F64ConvertUnsignI32,
  F64ConvertSignI64,
  F64ConvertUnsignI64,
  F64PromoteF32,
  I32ReinterpretF32,
  I64ReinterpretF64,
  F32ReinterpretI32,
  F64ReinterpretI64,

  RuntimeValue(ValueTypes),
}

pub enum TypeKind {
  Canonical(ValueTypes),
  Polymophic,
  Void,
}

impl Into<TypeKind> for &Inst {
  fn into(self) -> TypeKind {
    use self::Inst::*;
    match self {
      I32Const(_)
      | I32CountLeadingZero
      | I32CountTrailingZero
      | I32CountNonZero
      | I32Add
      | I32Sub
      | I32Mul
      | I32DivSign
      | I32DivUnsign
      | I32RemSign
      | I32RemUnsign
      | I32And
      | I32Or
      | I32Xor
      | I32ShiftLeft
      | I32ShiftRIghtSign
      | I32ShiftRightUnsign
      | I32RotateLeft
      | I32RotateRight
      | I32EqualZero
      | Equal
      | NotEqual
      | LessThanSign
      | LessThanUnsign
      | I32GreaterThanSign
      | I32GreaterThanUnsign
      | I32LessEqualSign
      | I32LessEqualUnsign
      | I32GreaterEqualSign
      | I32GreaterEqualUnsign
      | I32Load(_, _)
      | I32Load8Sign(_, _)
      | I32Load8Unsign(_, _)
      | I32Load16Sign(_, _)
      | I32Load16Unsign(_, _)
      | MemorySize
      | MemoryGrow
      | I32WrapI64
      | I32TruncUnsignF32
      | I32TruncUnsignF64
      | I32TruncSignF32
      | I32TruncSignF64
      | I32ReinterpretF32 => TypeKind::Canonical(ValueTypes::I32),
      I64Const(_)
      | I64CountLeadingZero
      | I64CountTrailingZero
      | I64CountNonZero
      | I64Add
      | I64Sub
      | I64Mul
      | I64DivSign
      | I64DivUnsign
      | I64RemSign
      | I64RemUnsign
      | I64And
      | I64Or
      | I64Xor
      | I64ShiftLeft
      | I64ShiftRightSign
      | I64ShiftRightUnsign
      | I64RotateLeft
      | I64RotateRight
      | I64EqualZero
      | I64Equal
      | I64NotEqual
      | I64LessThanSign
      | I64LessThanUnSign
      | I64GreaterThanSign
      | I64GreaterThanUnSign
      | I64LessEqualSign
      | I64LessEqualUnSign
      | I64GreaterEqualSign
      | I64GreaterEqualUnSign
      | I64Load(_, _)
      | I64Load8Sign(_, _)
      | I64Load8Unsign(_, _)
      | I64Load16Sign(_, _)
      | I64Load16Unsign(_, _)
      | I64Load32Sign(_, _)
      | I64Load32Unsign(_, _)
      | I64ExtendSignI32
      | I64ExtendUnsignI32
      | I64TruncSignF32
      | I64TruncUnsignF32
      | I64TruncSignF64
      | I64TruncUnsignF64
      | I64ReinterpretF64 => TypeKind::Canonical(ValueTypes::I64),
      F32Const(_)
      | F32Equal
      | F32NotEqual
      | F32LessThan
      | F32GreaterThan
      | F32LessEqual
      | F32GreaterEqual
      | F32Abs
      | F32Neg
      | F32Ceil
      | F32Floor
      | F32Trunc
      | F32Nearest
      | F32Sqrt
      | F32Add
      | F32Sub
      | F32Mul
      | F32Div
      | F32Min
      | F32Max
      | F32Copysign
      | F32Load(_, _)
      | F32ConvertSignI32
      | F32ConvertUnsignI32
      | F32ConvertSignI64
      | F32ConvertUnsignI64
      | F32DemoteF64
      | F32ReinterpretI32 => TypeKind::Canonical(ValueTypes::F32),
      F64Const(_)
      | F64Equal
      | F64NotEqual
      | F64LessThan
      | F64GreaterThan
      | F64LessEqual
      | F64GreaterEqual
      | F64Abs
      | F64Neg
      | F64Ceil
      | F64Floor
      | F64Trunc
      | F64Nearest
      | F64Sqrt
      | F64Add
      | F64Sub
      | F64Mul
      | F64Div
      | F64Min
      | F64Max
      | F64Copysign
      | F64Load(_, _)
      | F64ConvertSignI32
      | F64ConvertUnsignI32
      | F64ConvertSignI64
      | F64ConvertUnsignI64
      | F64PromoteF32
      | F64ReinterpretI64 => TypeKind::Canonical(ValueTypes::F64),
      Nop
      | DropInst
      | SetLocal(_)
      | SetGlobal(_)
      | I32Store(_, _)
      | I64Store(_, _)
      | F32Store(_, _)
      | F64Store(_, _)
      | I32Store8(_, _)
      | I32Store16(_, _)
      | I64Store8(_, _)
      | I64Store16(_, _)
      | I64Store32(_, _)
      | RuntimeValue(_)
      | Else
      | End => TypeKind::Void,
      Unreachable
      | Block(_)
      | If(_, _)
      | Loop
      | Br(_)
      | BrIf(_)
      | BrTable(_, _)
      | Return
      | Call(_)
      | CallIndirect(_)
      | GetGlobal(_)
      | Select
      | GetLocal(_)
      | TeeLocal(_) => TypeKind::Polymophic,
    }
  }
}
impl Inst {
  pub fn get_value_ext(&self) -> Values {
    use self::Inst::*;
    match self {
      I32Const(n) => Values::I32(*n),
      I64Const(n) => Values::I64(*n),
      F32Const(n) => Values::F32(*n),
      F64Const(n) => Values::F64(*n),
      _ => unreachable!("{:?}", self),
    }
  }
}

#[derive(PartialEq, Clone)]
pub struct Instructions {
  pub ptr: u32,
  expressions: Vec<Inst>,
  label_ptrs: Vec<u32>,
  table_addresses: Vec<u32>,
  types: Vec<Result<FunctionType>>,
}

impl fmt::Debug for Instructions {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "[{}][{}] labels[{:?}]",
      self
        .expressions
        .iter()
        .map(|p| format!("{:?}", p))
        .collect::<Vec<String>>()
        .join(", "),
      self.ptr,
      self.label_ptrs,
    )
  }
}

impl Instructions {
  pub fn new(
    expressions: Vec<Inst>,
    table_addresses: Vec<u32>,
    types: Vec<Result<FunctionType>>,
  ) -> Self {
    Instructions {
      ptr: 0,
      expressions,
      label_ptrs: vec![],
      table_addresses,
      types,
    }
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

  pub fn push_label(&mut self, ptr_of_label: u32) {
    self.label_ptrs.push(ptr_of_label)
  }

  pub fn pop_label(&mut self) -> Option<u32> {
    let ptr = self.label_ptrs.pop()?;
    Some(ptr)
  }

  pub fn jump_to(&mut self, ptr_of_label: u32) {
    self.ptr = ptr_of_label;
  }

  pub fn jump_to_label(&mut self, label: u32) {
    let mut label = label;
    let mut ptr_of_label = self
      .pop_label()
      .expect("When jump label excuted, at least one label should exists.");
    while label != 0 {
      label -= 1;
      ptr_of_label = self.pop_label().unwrap();
    }
    self.jump_to(ptr_of_label);
  }

  pub fn jump_to_last(&mut self) {
    let last = self.expressions.len();
    self.jump_to(last as u32);
  }

  pub fn get_table_address(&self) -> u32 {
    *self
      .table_addresses
      .get(0)
      .expect("Table address [0] not found")
  }
  pub fn get_type_at(&self, idx: u32) -> Option<&FunctionType> {
    match self.types.get(idx as usize) {
      Some(Ok(t)) => Some(t),
      _ => None,
    }
  }
}
