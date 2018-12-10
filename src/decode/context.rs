use code::ValueTypes;
use function::{FunctionInstance, FunctionType};
use global::GlobalInstance;
use inst::Inst;
use memory::MemoryInstance;
use store::Store;
use table::TableInstance;
use trap::{Result, Trap};

pub struct Context {
  function_instances: Vec<FunctionInstance>,
  memory_instances: Vec<MemoryInstance>,
  table_instances: Vec<TableInstance>,
  global_instances: Vec<GlobalInstance>,
}

impl Context {
  pub fn new(
    function_instances: Vec<FunctionInstance>,
    memory_instances: Vec<MemoryInstance>,
    table_instances: Vec<TableInstance>,
    global_instances: Vec<GlobalInstance>,
  ) -> Self {
    Context {
      function_instances,
      memory_instances,
      table_instances,
      global_instances,
    }
  }

  pub fn validate(self) -> Result<Store> {
    self
      .function_instances
      .iter()
      .map(|function_instance| {
        let function_type = function_instance.get_function_type().to_owned()?;
        let expect_return_type = function_type.get_return_types();
        let actual_return_type = self.reduction_instructions(function_instance, &function_type)?;
        if expect_return_type != &actual_return_type {
          return Err(Trap::TypeMismatch);
        }
        Ok(())
      })
      .collect::<Result<Vec<_>>>()?;

    Ok(Store::new(
      self.function_instances,
      self.memory_instances,
      self.table_instances,
      self.global_instances,
    ))
  }

  fn reduction_instructions(
    &self,
    function_instance: &FunctionInstance,
    function_type: &FunctionType,
  ) -> Result<Vec<ValueTypes>> {
    let (instructions, mut locals) = function_instance.call();
    let mut parameters = function_type.get_parameter_types().to_owned();
    parameters.append(&mut locals);
    self
      .reduction_instructions_internal(0, &instructions, &parameters)
      .map(|(_, return_type)| return_type)
  }
  fn reduction_instructions_internal(
    &self,
    mut inst_ptr: usize,
    instructions: &Vec<Inst>,
    locals: &Vec<ValueTypes>,
  ) -> Result<(usize, Vec<ValueTypes>)> {
    use self::Inst::*;
    let mut return_types: Vec<ValueTypes> = vec![];
    while inst_ptr < instructions.len() {
      match instructions.get(inst_ptr)? {
        If(_, _) => {
          // If
          let (next_inst_ptr, mut next_return_types) =
            self.reduction_instructions_internal(inst_ptr + 1, instructions, locals)?;
          inst_ptr = next_inst_ptr + 1;
          // Else
          let (next_inst_ptr, mut next_return_types) =
            self.reduction_instructions_internal(inst_ptr + 1, instructions, locals)?;

          println!(
            "return_types={:?} next_return_types={:?}",
            return_types, next_return_types
          );
          inst_ptr = next_inst_ptr;
          return_types.append(&mut next_return_types);
        }
        Block(_) | Loop => {
          unimplemented!();
        }
        End | Else => {
          let instruction = instructions.get(inst_ptr - 1)?;
          match instruction {
            I32Const(_) | I32ReinterpretF32 => return_types.push(ValueTypes::I32),
            I64Const(_) | I64ReinterpretF64 => return_types.push(ValueTypes::I64),
            F32Const(_) | F32ReinterpretI32 => return_types.push(ValueTypes::F32),
            F64Const(_) | F64ReinterpretI64 => return_types.push(ValueTypes::F64),
            Nop
            | End
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
            | I64Store32(_, _) => {}
            // NOTE: Returns polymophic type
            // Unreachable,
            // Br(_),
            // BrIf(_),
            // BrTable(Vec<_>, _),
            // Return,
            // Call(usize), // FIXME: Change to u32
            // CallIndirect(u32),
            // GetGlobal(u32),
            // Select,
            // RuntimeValue(_) | Else | End => unimplemented!("This type do not produce any types."),
            // _ => unimplemented!(),
            GetLocal(idx) | TeeLocal(idx) => {
              let ty = locals.get(*idx as usize);
              println!("ty={:?}", ty);
              if let Some(t) = ty {
                return_types.push(t.to_owned());
              };
            }
            _ => {}
          };
          break;
        }
        _ => { /* Nop */ }
      };
      inst_ptr += 1;
    }
    println!("return_types={:?}", return_types);
    Ok((inst_ptr, return_types))
  }
}

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
