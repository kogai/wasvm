use alloc::vec::Vec;
use core::convert::From;
use core::convert::Into;

#[derive(Debug, PartialEq, Clone)]
pub enum Isa {
  Reserved,
  Unreachable,
  Nop,
  Block,
  Loop,
  If,
  Else,
  End,
  Br,
  BrIf,
  BrTable,
  Return,
  Call,
  CallIndirect,
  Select,
  DropInst,
  I32Const,
  I64Const,
  F32Const,
  F64Const,
  GetLocal,
  TeeLocal,
  SetLocal,
  GetGlobal,
  SetGlobal,
  I32Load,
  I64Load,
  F32Load,
  F64Load,
  I32Load8Sign,
  I32Load8Unsign,
  I32Load16Sign,
  I32Load16Unsign,
  I64Load8Sign,
  I64Load8Unsign,
  I64Load16Sign,
  I64Load16Unsign,
  I64Load32Sign,
  I64Load32Unsign,
  I32Store,
  I64Store,
  F32Store,
  F64Store,
  I32Store8,
  I32Store16,
  I64Store8,
  I64Store16,
  I64Store32,
  MemorySize,
  MemoryGrow,
  I32CountLeadingZero,
  I32CountTrailingZero,
  I32CountNonZero,
  I32Add,
  I32Sub,
  I32Mul,
  I32WrapI64,
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
  I32Equal,
  I32NotEqual,
  I32LessThanSign,
  I32LessThanUnsign,
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
}

impl From<u8> for Isa {
  fn from(code: u8) -> Self {
    use self::Isa::*;
    match code {
      0x0 => Unreachable,
      0x1 => Nop,
      0x2 => Block,
      0x3 => Loop,
      0x4 => If,
      0x5 => Else,
      0x06 | 0x07 | 0x08 | 0x09 | 0x0A => Reserved,
      0x0b => End,
      0x0C => Br,
      0x0D => BrIf,
      0x0e => BrTable,
      0x0f => Return,
      0x10 => Call,
      0x11 => CallIndirect,
      0x12 | 0x13 | 0x14 | 0x15 | 0x16 | 0x17 | 0x18 | 0x19 => Reserved,
      0x1a => DropInst,
      0x1b => Select,
      0x20 => GetLocal,
      0x21 => SetLocal,
      0x22 => TeeLocal,
      0x23 => GetGlobal,
      0x24 => SetGlobal,
      0x25 | 0x26 | 0x27 => Reserved,
      0x28 => I32Load,
      0x29 => I64Load,
      0x2a => F32Load,
      0x2b => F64Load,
      0x2c => I32Load8Sign,
      0x2d => I32Load8Unsign,
      0x2e => I32Load16Sign,
      0x2f => I32Load16Unsign,
      0x30 => I64Load8Sign,
      0x31 => I64Load8Unsign,
      0x32 => I64Load16Sign,
      0x33 => I64Load16Unsign,
      0x34 => I64Load32Sign,
      0x35 => I64Load32Unsign,
      0x36 => I32Store,
      0x37 => I64Store,
      0x38 => F32Store,
      0x39 => F64Store,
      0x3a => I32Store8,
      0x3b => I32Store16,
      0x3c => I64Store8,
      0x3d => I64Store16,
      0x3e => I64Store32,
      0x3f => MemorySize,
      0x40 => MemoryGrow,
      0x41 => I32Const,
      0x42 => I64Const,
      0x43 => F32Const,
      0x44 => F64Const,
      0x45 => I32EqualZero,
      0x46 => I32Equal,
      0x47 => I32NotEqual,
      0x48 => I32LessThanSign,
      0x49 => I32LessThanUnsign,
      0x4a => I32GreaterThanSign,
      0x4b => I32GreaterThanUnsign,
      0x4c => I32LessEqualSign,
      0x4d => I32LessEqualUnsign,
      0x4e => I32GreaterEqualSign,
      0x4f => I32GreaterEqualUnsign,
      0x50 => I64EqualZero,
      0x51 => I64Equal,
      0x52 => I64NotEqual,
      0x53 => I64LessThanSign,
      0x54 => I64LessThanUnSign,
      0x55 => I64GreaterThanSign,
      0x56 => I64GreaterThanUnSign,
      0x57 => I64LessEqualSign,
      0x58 => I64LessEqualUnSign,
      0x59 => I64GreaterEqualSign,
      0x5a => I64GreaterEqualUnSign,
      0x5B => F32Equal,
      0x5C => F32NotEqual,
      0x5D => F32LessThan,
      0x5E => F32GreaterThan,
      0x5F => F32LessEqual,
      0x60 => F32GreaterEqual,
      0x61 => F64Equal,
      0x62 => F64NotEqual,
      0x63 => F64LessThan,
      0x64 => F64GreaterThan,
      0x65 => F64LessEqual,
      0x66 => F64GreaterEqual,
      0x67 => I32CountLeadingZero,
      0x68 => I32CountTrailingZero,
      0x69 => I32CountNonZero,
      0x6a => I32Add,
      0x6b => I32Sub,
      0x6c => I32Mul,
      0x6d => I32DivSign,
      0x6e => I32DivUnsign,
      0x6f => I32RemSign,
      0x70 => I32RemUnsign,
      0x71 => I32And,
      0x72 => I32Or,
      0x73 => I32Xor,
      0x74 => I32ShiftLeft,
      0x75 => I32ShiftRIghtSign,
      0x76 => I32ShiftRightUnsign,
      0x77 => I32RotateLeft,
      0x78 => I32RotateRight,
      0x79 => I64CountLeadingZero,
      0x7a => I64CountTrailingZero,
      0x7b => I64CountNonZero,
      0x7c => I64Add,
      0x7d => I64Sub,
      0x7e => I64Mul,
      0x7f => I64DivSign,
      0x80 => I64DivUnsign,
      0x81 => I64RemSign,
      0x82 => I64RemUnsign,
      0x83 => I64And,
      0x84 => I64Or,
      0x85 => I64Xor,
      0x86 => I64ShiftLeft,
      0x87 => I64ShiftRightSign,
      0x88 => I64ShiftRightUnsign,
      0x89 => I64RotateLeft,
      0x8a => I64RotateRight,
      0x8b => F32Abs,
      0x8c => F32Neg,
      0x8d => F32Ceil,
      0x8e => F32Floor,
      0x8f => F32Trunc,
      0x90 => F32Nearest,
      0x91 => F32Sqrt,
      0x92 => F32Add,
      0x93 => F32Sub,
      0x94 => F32Mul,
      0x95 => F32Div,
      0x96 => F32Min,
      0x97 => F32Max,
      0x98 => F32Copysign,
      0x99 => F64Abs,
      0x9a => F64Neg,
      0x9b => F64Ceil,
      0x9c => F64Floor,
      0x9d => F64Trunc,
      0x9e => F64Nearest,
      0x9f => F64Sqrt,
      0xa0 => F64Add,
      0xa1 => F64Sub,
      0xa2 => F64Mul,
      0xa3 => F64Div,
      0xa4 => F64Min,
      0xa5 => F64Max,
      0xa6 => F64Copysign,
      0xa7 => I32WrapI64,
      0xa8 => I32TruncSignF32,
      0xa9 => I32TruncUnsignF32,
      0xaa => I32TruncSignF64,
      0xab => I32TruncUnsignF64,
      0xac => I64ExtendSignI32,
      0xad => I64ExtendUnsignI32,
      0xae => I64TruncSignF32,
      0xaf => I64TruncUnsignF32,
      0xb0 => I64TruncSignF64,
      0xb1 => I64TruncUnsignF64,
      0xb2 => F32ConvertSignI32,
      0xb3 => F32ConvertUnsignI32,
      0xb4 => F32ConvertSignI64,
      0xb5 => F32ConvertUnsignI64,
      0xb6 => F32DemoteF64,
      0xb7 => F64ConvertSignI32,
      0xb8 => F64ConvertUnsignI32,
      0xb9 => F64ConvertSignI64,
      0xba => F64ConvertUnsignI64,
      0xbb => F64PromoteF32,
      0xbc => I32ReinterpretF32,
      0xbd => I64ReinterpretF64,
      0xbe => F32ReinterpretI32,
      0xbf => F64ReinterpretI64,
      x => unreachable!("Code {:x?} does not supported yet.", x),
    }
  }
}

impl Into<u8> for Isa {
  fn into(self) -> u8 {
    use self::Isa::*;
    match self {
      Reserved => unreachable!(),
      Unreachable => 0x0,
      Nop => 0x1,
      Block => 0x2,
      Loop => 0x3,
      If => 0x4,
      Else => 0x5,
      End => 0x0b,
      Br => 0x0C,
      BrIf => 0x0D,
      BrTable => 0x0e,
      Return => 0x0f,
      Call => 0x10,
      CallIndirect => 0x11,
      DropInst => 0x1a,
      Select => 0x1b,
      GetLocal => 0x20,
      SetLocal => 0x21,
      TeeLocal => 0x22,
      GetGlobal => 0x23,
      SetGlobal => 0x24,
      I32Load => 0x28,
      I64Load => 0x29,
      F32Load => 0x2a,
      F64Load => 0x2b,
      I32Load8Sign => 0x2c,
      I32Load8Unsign => 0x2d,
      I32Load16Sign => 0x2e,
      I32Load16Unsign => 0x2f,
      I64Load8Sign => 0x30,
      I64Load8Unsign => 0x31,
      I64Load16Sign => 0x32,
      I64Load16Unsign => 0x33,
      I64Load32Sign => 0x34,
      I64Load32Unsign => 0x35,
      I32Store => 0x36,
      I64Store => 0x37,
      F32Store => 0x38,
      F64Store => 0x39,
      I32Store8 => 0x3a,
      I32Store16 => 0x3b,
      I64Store8 => 0x3c,
      I64Store16 => 0x3d,
      I64Store32 => 0x3e,
      MemorySize => 0x3f,
      MemoryGrow => 0x40,
      I32Const => 0x41,
      I64Const => 0x42,
      F32Const => 0x43,
      F64Const => 0x44,
      I32EqualZero => 0x45,
      I32Equal => 0x46,
      I32NotEqual => 0x47,
      I32LessThanSign => 0x48,
      I32LessThanUnsign => 0x49,
      I32GreaterThanSign => 0x4a,
      I32GreaterThanUnsign => 0x4b,
      I32LessEqualSign => 0x4c,
      I32LessEqualUnsign => 0x4d,
      I32GreaterEqualSign => 0x4e,
      I32GreaterEqualUnsign => 0x4f,
      I64EqualZero => 0x50,
      I64Equal => 0x51,
      I64NotEqual => 0x52,
      I64LessThanSign => 0x53,
      I64LessThanUnSign => 0x54,
      I64GreaterThanSign => 0x55,
      I64GreaterThanUnSign => 0x56,
      I64LessEqualSign => 0x57,
      I64LessEqualUnSign => 0x58,
      I64GreaterEqualSign => 0x59,
      I64GreaterEqualUnSign => 0x5a,
      F32Equal => 0x5B,
      F32NotEqual => 0x5C,
      F32LessThan => 0x5D,
      F32GreaterThan => 0x5E,
      F32LessEqual => 0x5F,
      F32GreaterEqual => 0x60,
      F64Equal => 0x61,
      F64NotEqual => 0x62,
      F64LessThan => 0x63,
      F64GreaterThan => 0x64,
      F64LessEqual => 0x65,
      F64GreaterEqual => 0x66,
      I32CountLeadingZero => 0x67,
      I32CountTrailingZero => 0x68,
      I32CountNonZero => 0x69,
      I32Add => 0x6a,
      I32Sub => 0x6b,
      I32Mul => 0x6c,
      I32DivSign => 0x6d,
      I32DivUnsign => 0x6e,
      I32RemSign => 0x6f,
      I32RemUnsign => 0x70,
      I32And => 0x71,
      I32Or => 0x72,
      I32Xor => 0x73,
      I32ShiftLeft => 0x74,
      I32ShiftRIghtSign => 0x75,
      I32ShiftRightUnsign => 0x76,
      I32RotateLeft => 0x77,
      I32RotateRight => 0x78,
      I64CountLeadingZero => 0x79,
      I64CountTrailingZero => 0x7a,
      I64CountNonZero => 0x7b,
      I64Add => 0x7c,
      I64Sub => 0x7d,
      I64Mul => 0x7e,
      I64DivSign => 0x7f,
      I64DivUnsign => 0x80,
      I64RemSign => 0x81,
      I64RemUnsign => 0x82,
      I64And => 0x83,
      I64Or => 0x84,
      I64Xor => 0x85,
      I64ShiftLeft => 0x86,
      I64ShiftRightSign => 0x87,
      I64ShiftRightUnsign => 0x88,
      I64RotateLeft => 0x89,
      I64RotateRight => 0x8a,
      F32Abs => 0x8b,
      F32Neg => 0x8c,
      F32Ceil => 0x8d,
      F32Floor => 0x8e,
      F32Trunc => 0x8f,
      F32Nearest => 0x90,
      F32Sqrt => 0x91,
      F32Add => 0x92,
      F32Sub => 0x93,
      F32Mul => 0x94,
      F32Div => 0x95,
      F32Min => 0x96,
      F32Max => 0x97,
      F32Copysign => 0x98,
      F64Abs => 0x99,
      F64Neg => 0x9a,
      F64Ceil => 0x9b,
      F64Floor => 0x9c,
      F64Trunc => 0x9d,
      F64Nearest => 0x9e,
      F64Sqrt => 0x9f,
      F64Add => 0xa0,
      F64Sub => 0xa1,
      F64Mul => 0xa2,
      F64Div => 0xa3,
      F64Min => 0xa4,
      F64Max => 0xa5,
      F64Copysign => 0xa6,
      I32WrapI64 => 0xa7,
      I32TruncSignF32 => 0xa8,
      I32TruncUnsignF32 => 0xa9,
      I32TruncSignF64 => 0xaa,
      I32TruncUnsignF64 => 0xab,
      I64ExtendSignI32 => 0xac,
      I64ExtendUnsignI32 => 0xad,
      I64TruncSignF32 => 0xae,
      I64TruncUnsignF32 => 0xaf,
      I64TruncSignF64 => 0xb0,
      I64TruncUnsignF64 => 0xb1,
      F32ConvertSignI32 => 0xb2,
      F32ConvertUnsignI32 => 0xb3,
      F32ConvertSignI64 => 0xb4,
      F32ConvertUnsignI64 => 0xb5,
      F32DemoteF64 => 0xb6,
      F64ConvertSignI32 => 0xb7,
      F64ConvertUnsignI32 => 0xb8,
      F64ConvertSignI64 => 0xb9,
      F64ConvertUnsignI64 => 0xba,
      F64PromoteF32 => 0xbb,
      I32ReinterpretF32 => 0xbc,
      I64ReinterpretF64 => 0xbd,
      F32ReinterpretI32 => 0xbe,
      F64ReinterpretI64 => 0xbf,
    }
  }
}

#[allow(dead_code)]
pub enum ComposedCode {
  Code(Isa),
  Byte(u8),
}

pub fn into_vec_u8(this: &[ComposedCode]) -> Vec<u8> {
  this
    .iter()
    .map(|x| match x {
      ComposedCode::Code(code) => Isa::into(code.clone()),
      ComposedCode::Byte(byte) => *byte,
    })
    .collect()
}

impl Isa {
  pub fn is_else_or_end(code: Option<u8>) -> bool {
    match code {
      Some(0x5) | Some(0x0b) => true,
      _ => false,
    }
  }
}
