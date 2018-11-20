#[derive(Debug, PartialEq, Clone)]
pub enum Inst {
  I32Const(i32),
  I64Const(i64),
  // FIXME: Change to u32
  GetLocal(usize),
  SetLocal(usize),
  TeeLocal(usize),
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
  // FIXME: Change to u32
  Call(usize),
  I32EqualZero,
  Equal,
  NotEqual,
  LessThanSign,
  LessThanUnsign,
  GreaterThanSign,
  GreaterThanUnsign,
  I32GreaterThanUnsign,
  I32LessEqualSign,
  I32LessEqualUnsign,
  I32GreaterEqualSign,
  I32GreaterEqualUnsign,
  If(Vec<Inst>, Vec<Inst>),
  Select,
  Return,
  TypeI32,
  TypeEmpty,
  I64ExtendUnsignI32,
  I64Mul,
  I64And,
  I64ShiftRightUnsign,
  I32WrapI64,
}

pub enum Trap {
  DivisionOverflow,
  DivisionByZero,
}