use code::ValueTypes;
use inst::Inst;
use std::fmt;
use trap::{Result, Trap};

#[derive(PartialEq, Clone)]
pub struct FunctionType {
  parameters: Vec<ValueTypes>,
  returns: Vec<ValueTypes>,
}

impl fmt::Debug for FunctionType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "({}) -> ({})",
      self
        .parameters
        .iter()
        .map(|p| format!("{:?}", p))
        .collect::<Vec<String>>()
        .join(", "),
      self
        .returns
        .iter()
        .map(|p| format!("{:?}", p))
        .collect::<Vec<String>>()
        .join(", "),
    )
  }
}

impl FunctionType {
  pub fn new(parameters: Vec<ValueTypes>, returns: Vec<ValueTypes>) -> Self {
    FunctionType {
      parameters,
      returns,
    }
  }

  pub fn get_return_count(&self) -> u32 {
    self.returns.len() as u32
  }
}

impl FunctionType {
  pub fn get_arity(&self) -> u32 {
    self.parameters.len() as u32
  }
  fn get_return(&self) -> Option<ValueTypes> {
    self.returns.first().map(|x| x.to_owned())
  }
}

#[derive(PartialEq, Clone)]
pub struct FunctionInstance {
  export_name: Option<String>,
  pub function_type: Result<FunctionType>,
  pub locals: Vec<ValueTypes>,
  type_idex: u32,
  body: Vec<Inst>,
}

impl fmt::Debug for FunctionInstance {
  // TODO: Consider also to present instructions.
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let name = match self.export_name {
      Some(ref n) => n,
      _ => "_",
    };
    let function_type = match self.function_type {
      Ok(ref f) => format!("{:?}", f),
      Err(ref err) => format!("{:?}", err),
    };
    write!(f, "[{}] {}: {}", self.type_idex, name, function_type)
  }
}

impl FunctionInstance {
  fn reduction_instructions(instructions: &Vec<Inst>) -> Result<Option<ValueTypes>> {
    let mut return_types: Vec<ValueTypes> = vec![];
    let mut inst_ptr = 0;
    use self::Inst::*;
    while inst_ptr < instructions.len() {
      // NOTE: Peek next
      if let Some(End) = instructions.get(inst_ptr + 1) {
        let instruction = instructions.get(inst_ptr)?;
        match instruction {
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
          | I32TruncUnsignF64
          | I32ReinterpretF32 => return_types.push(ValueTypes::I32),
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
          | I64ReinterpretF64 => return_types.push(ValueTypes::I64),
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
          | F32ReinterpretI32 => return_types.push(ValueTypes::F32),
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
          | F64ReinterpretI64 => return_types.push(ValueTypes::F64),
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
          | I64Store32(_, _) => {
            unimplemented!(
              "When enter this pattern, it's may time to implements case for not return anything."
            );
          }
          // NOTE: Returns polymophic type
          // Unreachable,
          // Block(u32),
          // Loop,
          // If(u32, u32),
          // Br(u32),
          // BrIf(u32),
          // BrTable(Vec<u32>, u32),
          // Return,
          // Call(usize), // FIXME: Change to u32
          // CallIndirect(u32),
          // GetLocal(u32),
          // TeeLocal(u32),
          // GetGlobal(u32),
          // Select,
          RuntimeValue(_) | Else | End => unimplemented!("This type do not produce any types."),
          _ => unimplemented!(),
        };
      };
      inst_ptr += 1;
    }
    let mut return_type_ptr = 0;
    let mut return_type = None;
    while return_type_ptr < return_types.len() {
      let current = return_types.get(return_type_ptr);
      let next = return_types.get(return_type_ptr + 1);
      match (current, next) {
        (Some(t1), Some(t2)) => {
          if t1 != t2 {
            return Err(Trap::TypeMismatch);
          };
          return_type = Some(t1);
        }
        (Some(t), None) => {
          return_type = Some(t);
        }
        _ => unreachable!(),
      };
      return_type_ptr += 1;
    }
    Ok(return_type.map(|x| x.to_owned()))
  }

  pub fn new(
    export_name: Option<String>,
    function_type: Result<FunctionType>,
    locals: Vec<ValueTypes>,
    type_idex: u32,
    body: Vec<Inst>,
  ) -> Result<Self> {
    let expect_return_type = function_type.to_owned()?.get_return();
    let actual_return_type = FunctionInstance::reduction_instructions(&body)?;
    if expect_return_type != actual_return_type {
      return Err(Trap::TypeMismatch);
    }
    Ok(FunctionInstance {
      export_name,
      function_type,
      locals,
      type_idex,
      body,
    })
  }

  pub fn call(&self) -> (Vec<Inst>, Vec<ValueTypes>) {
    (self.body.to_owned(), self.locals.to_owned())
  }

  pub fn get_arity(&self) -> u32 {
    match self.function_type {
      Ok(ref f) => f.get_arity(),
      _ => 0,
    }
  }

  pub fn find(&self, key: &str) -> bool {
    // FIXME: When using function_index, we might get exported function by O(1).
    match &self.export_name {
      Some(name) => name.as_str() == key,
      _ => false,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  macro_rules! impl_test_validate {
    ($fn_name:ident, $return_type: expr, $instructions: expr) => {
      #[test]
      fn $fn_name() {
        use self::Inst::*;
        let actual = FunctionInstance::new(
          None,
          Ok(FunctionType::new(vec![], vec![$return_type])),
          vec![],
          0,
          $instructions,
        );
        assert_eq!(actual, Err(Trap::TypeMismatch));
      }
    };
    ($fn_name:ident, $return_type: expr, $locals: expr, $instructions: expr) => {
      #[test]
      fn $fn_name() {
        use self::Inst::*;
        let actual = FunctionInstance::new(
          None,
          Ok(FunctionType::new(vec![], vec![$return_type])),
          $locals,
          0,
          $instructions,
        );
        assert_eq!(actual, Err(Trap::TypeMismatch));
      }
    };
  }

  impl_test_validate!(
    validate_return_type,
    ValueTypes::I32,
    vec![I64Const(0), End]
  );
  impl_test_validate!(
    validate_return_locals,
    ValueTypes::I32,
    vec![ValueTypes::I64],
    vec![GetLocal(0), End]
  );
}
