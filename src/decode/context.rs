use super::section::Section;
use trap::Result;

pub struct Context<'a> {
  module: &'a Section,
}

impl<'a> Context<'a> {
  pub fn new(module: &'a Section) -> Self {
    Context { module }
  }

  // FIXME: Skip type validation until ready.
  pub fn validate(&self) -> Result<()> {
    unimplemented!(
      "Type system(Also called as `validation`) not implemented yet.\n{:?}",
      self.module
    );
  }

  // fn reduction_instructions(
  //   &self,
  //   _function_instance: &FunctionInstance,
  //   _function_type: &FunctionType,
  // ) -> Result<Vec<ValueTypes>> {
  //   // let (expressions, mut locals) = function_instance.call();
  //   // let mut parameters = function_type.get_parameter_types().to_owned();
  //   // parameters.append(&mut locals);
  //   // let mut instructions = Instructions::new(expressions, vec![0]);
  //   // let return_type = self.reduction_instructions_internal(&mut instructions, &parameters)?;
  //   // Ok(vec![return_type])
  //   unimplemented!();
  // }

  // NOTE: Currently, WASM specification supposes to single return value.
  // fn reduction_instructions_internal(
  //   &self,
  //   instructions: &mut Frame,
  //   _locals: &Vec<ValueTypes>,
  // ) -> Result<ValueTypes> {
  //   let mut _return_type: ValueTypes;
  //   while !instructions.is_next_end_or_else() {
  //     let instruction = instructions.pop_ref()?;
  //     match instruction.into() {
  //       TypeKind::Canonical(_ty) => {
  //         println!("instruction={:?}", instruction);
  //         unimplemented!();
  //       }
  //       TypeKind::Polymophic => {
  //         println!("instruction={:?}", instruction);
  //         unimplemented!();
  //       }
  //       TypeKind::Void => {}
  //     }
  //   }
  //   unimplemented!();
  // }
}

/*
#[cfg(test)]
mod tests {
  use super::*;
  use code::ValueTypes;
  use function::FunctionType;
  use inst::Inst;

  #[test]
  fn test_validate_return_type() {
    let export_name = None;
    let function_type = Ok(FunctionType::new(vec![], vec![ValueTypes::I32]));
    let locals = vec![];
    let type_idx = 0;
    let body = vec![Inst::I64Const(0), Inst::End];
    let actual = Context::new(
      vec![FunctionInstance::new(
        export_name,
        function_type,
        locals,
        type_idx,
        body,
      )],
      vec![],
      vec![],
      vec![],
    )
    .validate();
    assert_eq!(actual.unwrap_err(), Trap::TypeMismatch);
  }
  #[test]
  fn test_validate_return_if() {
    use self::Inst::*;
    let export_name = None;
    let function_type = Ok(FunctionType::new(vec![], vec![ValueTypes::I32]));
    let locals = vec![];
    let type_idx = 0;
    let body = vec![
      If(0, 0),
      RuntimeValue(ValueTypes::I32),
      I64Const(0),
      Else,
      I64Const(0),
      If(0, 0),
      RuntimeValue(ValueTypes::I32),
      F32Const(0.0),
      Else,
      F32Const(0.0),
      End,
      End,
      End,
    ];
    let actual = Context::new(
      vec![FunctionInstance::new(
        export_name,
        function_type,
        locals,
        type_idx,
        body,
      )],
      vec![],
      vec![],
      vec![],
    )
    .validate();
    assert_eq!(actual.unwrap_err(), Trap::TypeMismatch);
  }
}
*/
