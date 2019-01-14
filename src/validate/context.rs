use super::error::{Result, TypeError};
use alloc::vec::Vec;
use decode::Section;
use function::FunctionType;
use inst::{Indice, Inst};
use module::{FUNCTION_DESCRIPTOR, GLOBAL_DESCRIPTOR, MEMORY_DESCRIPTOR, TABLE_DESCRIPTOR};
use trap::Trap;
use value::Values;
use value_type::ValueTypes;

struct Function<'a> {
  function_type: &'a FunctionType,
  locals: &'a Vec<ValueTypes>,
  body: &'a Vec<Inst>,
}

impl<'a> Function<'a> {
  fn new(
    function_type: &'a FunctionType,
    locals: &'a Vec<ValueTypes>,
    body: &'a Vec<Inst>,
  ) -> Function<'a> {
    Function {
      function_type,
      locals,
      body,
    }
  }
}

pub struct Context<'a> {
  module: &'a Section,
  function_types: &'a Vec<FunctionType>,
  functions: Vec<Function<'a>>,
  //  exports: ExternalInterfaces,
  //  codes: Vec<Result<(Vec<Inst>, Vec<ValueTypes>)>>,
  //  datas: Vec<Data>,
  //  limits: Vec<Limit>,
  //  tables: Vec<TableType>,
  //  globals: Vec<(GlobalType, Vec<Inst>)>,
  //  elements: Vec<Element>,
  //  customs: Vec<(String, Vec<u8>)>,
  //  imports: ExternalInterfaces,
  //  start: Option<u32>,
}

impl<'a> Context<'a> {
  pub fn new(module: &'a Section) -> Result<Self> {
    Ok(Context {
      module,
      function_types: &module.function_types,
      functions: module
        .codes
        .iter()
        .enumerate()
        .map(|(idx, code)| {
          let idx = module.functions.get(idx).map(|n| Indice::from(*n))?;
          let function_type = module.function_types.get(idx.to_usize())?;
          let (body, locals) = match code {
            Ok((body, locals)) => Ok((body, locals)),
            Err(ref err) => Err(TypeError::Trap(err.to_owned())),
          }?;
          Ok(Function::new(function_type, locals, body))
        })
        .collect::<Result<Vec<_>>>()?,
    })
  }

  fn validate_function_types(&self) -> Result<()> {
    for fy in self.function_types.iter() {
      if fy.returns().len() > 1 {
        return Err(TypeError::TypeMismatch);
      }
    }
    Ok(())
  }

  pub fn validate(&self) -> Result<()> {
    // let grouped_imports = self.module.imports.group_by_kind();
    // let imports_function = grouped_imports.get(&FUNCTION_DESCRIPTOR)?;
    // let imports_table = grouped_imports.get(&TABLE_DESCRIPTOR)?;
    // let imports_memory = grouped_imports.get(&MEMORY_DESCRIPTOR)?;
    // let imports_global = grouped_imports.get(&GLOBAL_DESCRIPTOR)?;
    self.validate_function_types();

    // let mut internal_function_instances = Section::function_instances(
    //   &self.module.function_types,
    //   &self.module.functions,
    //   &self.module.exports,
    //   self.module.codes,
    // )?;

    // let mut function_instances =
    //   Section::external_function_instances(&function_types, &imports_function, &external_modules)?;
    // function_instances.append(&mut internal_function_instances);

    // let global_instances =
    //   GlobalInstances::new_with_external(globals, &exports, &imports_global, &external_modules)?;

    unimplemented!(
      "Type system(Also called as `validation`) not implemented yet.\n{:#?}",
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
