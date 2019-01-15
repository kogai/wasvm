use super::error::{Result, TypeError};
use alloc::vec::Vec;
use core::cell::RefCell;
use decode::Section;
use function::FunctionType;
use inst::{Indice, Inst};
// use module::{FUNCTION_DESCRIPTOR, GLOBAL_DESCRIPTOR, MEMORY_DESCRIPTOR, TABLE_DESCRIPTOR};
// use trap::Trap;
// use value::Values;
use value_type::ValueTypes;

type ResultType = [ValueTypes; 1];

struct Stack {
  locals: Vec<ValueTypes>,
  labels: Vec<ResultType>,
  return_type: ResultType,
}

impl Default for Stack {
  fn default() -> Self {
    Stack {
      locals: Vec::new(),
      labels: Vec::new(),
      return_type: [ValueTypes::Empty; 1],
    }
  }
}

struct Function<'a> {
  function_type: &'a FunctionType,
  locals: &'a Vec<ValueTypes>,
  body: &'a Vec<Inst>,
  stack: RefCell<Stack>,
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
      stack: Default::default(),
    }
  }
}

pub struct Context<'a> {
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

  fn validate_functions(&self) -> Result<()> {
    for f in self.functions.iter() {
      self.validate_function(f)?;
    }
    Ok(())
  }

  fn validate_function(&self, function: &Function) -> Result<()> {
    // for fy in self.function_types.iter() {
    //   if fy.returns().len() > 1 {
    //     return Err(TypeError::TypeMismatch);
    //   }
    // }
    Ok(())
  }

  pub fn validate(&self) -> Result<()> {
    // let grouped_imports = self.module.imports.group_by_kind();
    // let imports_function = grouped_imports.get(&FUNCTION_DESCRIPTOR)?;
    // let imports_table = grouped_imports.get(&TABLE_DESCRIPTOR)?;
    // let imports_memory = grouped_imports.get(&MEMORY_DESCRIPTOR)?;
    // let imports_global = grouped_imports.get(&GLOBAL_DESCRIPTOR)?;
    self.validate_function_types()?;
    self.validate_functions()?;

    // let global_instances =
    //   GlobalInstances::new_with_external(globals, &exports, &imports_global, &external_modules)?;

    // unimplemented!(
    //   "Type system(Also called as `validation`) not implemented yet.\n{:#?}",
    //   self.module
    // );
    Ok(())
  }
}
