use decode::{Element, ElementType, TableInstance, TableType};
use function::{FunctionInstance, FunctionType};
use global::{GlobalInstance, GlobalType};
use inst::Inst;
use memory::{Limit, MemoryInstance};
use module::ExternalModule;
use value::Values;
use value_type::ValueTypes;

pub fn create_spectest() -> ExternalModule {
  ExternalModule::new(
    vec![
      FunctionInstance::new(
        Some("print".to_owned()),
        FunctionType::new(vec![], vec![]),
        vec![],
        0,
        vec![Inst::End],
      ),
      // 4
      FunctionInstance::new(
        Some("print_i32".to_owned()),
        FunctionType::new(vec![ValueTypes::I32], vec![]),
        vec![],
        0,
        vec![Inst::End],
      ),
      // 5
      FunctionInstance::new(
        Some("print_i32_f32".to_owned()),
        FunctionType::new(vec![ValueTypes::I32, ValueTypes::F32], vec![]),
        vec![],
        0,
        vec![Inst::End],
      ),
      // 6
      FunctionInstance::new(
        Some("print_f64_f64".to_owned()),
        FunctionType::new(vec![ValueTypes::F64, ValueTypes::F64], vec![]),
        vec![],
        0,
        vec![Inst::End],
      ),
      // 2
      FunctionInstance::new(
        Some("print_f32".to_owned()),
        FunctionType::new(vec![ValueTypes::F32], vec![]),
        vec![],
        0,
        vec![Inst::End],
      ),
      // 3
      FunctionInstance::new(
        Some("print_f64".to_owned()),
        FunctionType::new(vec![ValueTypes::F64], vec![]),
        vec![],
        0,
        vec![Inst::End],
      ),
    ],
    vec![],
    // MemoryInstances
    vec![MemoryInstance::new(
      vec![],
      Limit::HasUpperLimit(1, 2),
      Some("memory".to_owned()),
      &vec![],
    )
    .unwrap()],
    // TableInstances
    vec![TableInstance::new(
      Some(&TableType::new(
        ElementType::AnyFunc,
        Limit::HasUpperLimit(0, 20),
        // Limit::HasUpperLimit(10, 20),
      )),
      Element::new(0, vec![Inst::I32Const(10)], vec![10]),
      Some("table".to_owned()),
    )],
    // GlobalInstances
    vec![
      GlobalInstance::new(
        GlobalType::Var(ValueTypes::I32),
        Values::I32(666),
        Some("global_i32".to_owned()),
      ),
      GlobalInstance::new(
        GlobalType::Var(ValueTypes::F32),
        Values::F32(666.6),
        Some("global_f32".to_owned()),
      ),
      GlobalInstance::new(
        GlobalType::Var(ValueTypes::F64),
        Values::F64(666.6),
        Some("global_f64".to_owned()),
      ),
    ],
  )
}
