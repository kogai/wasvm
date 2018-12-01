use code::ValueTypes;

#[derive(Debug, PartialEq, Clone)]
pub enum Inst {
  Unreachable,
  Nop,
  Block,
  Loop,
  If,
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

#[derive(Debug, PartialEq, Clone)]
pub struct Instructions {
  index: usize,
  expressions: Vec<Inst>,
}

impl Instructions {
  pub fn new(expressions: Vec<Inst>) -> Self {
    Instructions {
      index: 0,
      expressions,
    }
  }
  pub fn peek(&self) -> Option<&Inst> {
    self.expressions.get(self.index)
  }
  pub fn pop(&mut self) -> Option<Inst> {
    let head = self.expressions.get(self.index).map(|x| x.clone());
    self.index += 1;
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
  pub fn skip_until_end_or_else(&mut self) {
    while !self.is_next_end_or_else() {
      match self.peek() {
        Some(Inst::If) => {
          let _ = self.pop();
          self.skip_until_end_or_else();
          if self.is_next_else() {
            let _ = self.pop();
            self.skip_until_end_or_else();
          } else {
            let _ = self.pop();
          }
        }
        _ => {
          let _ = self.pop();
        }
      }
    }
  }
}
