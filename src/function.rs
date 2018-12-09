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
        let instruction = instructions.get(inst_ptr);
        match instruction {
          Some(I32Const(_)) => return_types.push(ValueTypes::I32),
          Some(I64Const(_)) => return_types.push(ValueTypes::I64),
          _ => { /* Nop */ }
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
