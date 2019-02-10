#[cfg(not(test))]
use alloc::prelude::*;
use decode::{ElementType, TableType};
use function::{FunctionInstance, FunctionType};
use global::{GlobalInstance, GlobalInstances, GlobalType};
use isa::{into_vec_u8, ComposedCode, Isa};
use memory::{Limit, MemoryInstance};
use module::ExternalModule;
use table::TableInstance;
use value::Values;
use value_type::{TYPE_F32, TYPE_F64, TYPE_I32};

pub fn create_spectest() -> ExternalModule {
  ExternalModule::new(
    // FIXME: Change to definition as Host function.
    vec![
      FunctionInstance::new(
        Some("print".to_owned()),
        FunctionType::new(vec![], vec![]),
        vec![],
        into_vec_u8(&[ComposedCode::Code(Isa::End)]),
      ),
      // 4
      FunctionInstance::new(
        Some("print_i32".to_owned()),
        FunctionType::new(vec![TYPE_I32], vec![]),
        vec![],
        into_vec_u8(&[ComposedCode::Code(Isa::End)]),
      ),
      // 5
      FunctionInstance::new(
        Some("print_i32_f32".to_owned()),
        FunctionType::new(vec![TYPE_I32, TYPE_F32], vec![]),
        vec![],
        into_vec_u8(&[ComposedCode::Code(Isa::End)]),
      ),
      // 6
      FunctionInstance::new(
        Some("print_f64_f64".to_owned()),
        FunctionType::new(vec![TYPE_F64, TYPE_F64], vec![]),
        vec![],
        into_vec_u8(&[ComposedCode::Code(Isa::End)]),
      ),
      // 2
      FunctionInstance::new(
        Some("print_f32".to_owned()),
        FunctionType::new(vec![TYPE_F32], vec![]),
        vec![],
        into_vec_u8(&[ComposedCode::Code(Isa::End)]),
      ),
      // 3
      FunctionInstance::new(
        Some("print_f64".to_owned()),
        FunctionType::new(vec![TYPE_F64], vec![]),
        vec![],
        into_vec_u8(&[ComposedCode::Code(Isa::End)]),
      ),
    ],
    vec![],
    // MemoryInstances
    vec![MemoryInstance::new(
      vec![],
      Limit::HasUpperLimit(1, 2),
      Some("memory".to_owned()),
      &GlobalInstances::empty(),
    )
    .unwrap()],
    // TableInstances
    vec![TableInstance::new(
      vec![],
      TableType::new(ElementType::AnyFunc, Limit::HasUpperLimit(10, 20)),
      Some("table".to_owned()),
      &GlobalInstances::empty(),
      &[],
    )
    .unwrap()],
    // GlobalInstances
    vec![
      GlobalInstance::new(
        GlobalType::Const(TYPE_I32),
        Values::I32(666),
        Some("global_i32".to_owned()),
      ),
      GlobalInstance::new(
        GlobalType::Const(TYPE_F32),
        Values::F32(666.6),
        Some("global_f32".to_owned()),
      ),
      GlobalInstance::new(
        GlobalType::Const(TYPE_F64),
        Values::F64(666.6),
        Some("global_f64".to_owned()),
      ),
    ],
  )
}
