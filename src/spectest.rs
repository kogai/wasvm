use function::{FunctionInstance, FunctionType};
use inst::Inst;
use module::ExternalModule;
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
      FunctionInstance::new(
        Some("print_i32".to_owned()),
        FunctionType::new(vec![ValueTypes::I32], vec![]),
        vec![],
        0,
        vec![Inst::End],
      ),
      FunctionInstance::new(
        Some("print_i32_f32".to_owned()),
        FunctionType::new(vec![ValueTypes::I32, ValueTypes::F32], vec![]),
        vec![],
        0,
        vec![Inst::End],
      ),
      FunctionInstance::new(
        Some("print_f64_f64".to_owned()),
        FunctionType::new(vec![ValueTypes::F64, ValueTypes::F64], vec![]),
        vec![],
        0,
        vec![Inst::End],
      ),
      FunctionInstance::new(
        Some("print_f32".to_owned()),
        FunctionType::new(vec![ValueTypes::F32], vec![]),
        vec![],
        0,
        vec![Inst::End],
      ),
      FunctionInstance::new(
        Some("print_f64".to_owned()),
        FunctionType::new(vec![ValueTypes::F64], vec![]),
        vec![],
        0,
        vec![Inst::End],
      ),
    ],
    vec![],
    vec![],
    vec![],
    vec![],
  )
}
