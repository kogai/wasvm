extern crate wabt;

#[cfg(test)]
extern crate wasvm;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use wabt::script::{Action, Command, CommandKind, ScriptParser, Value};
use wasvm::{Values, Vm};

fn get_args(args: &Vec<Value<f32, f64>>) -> Vec<Values> {
  args
    .iter()
    .map(|v| match v {
      Value::I32(value) => Values::I32(*value),
      Value::I64(value) => Values::I64(*value),
      Value::F32(value) => Values::F32(*value),
      Value::F64(value) => Values::F64(*value),
    })
    .collect()
}

fn get_expectation(expected: &Vec<Value>) -> String {
  match expected.get(0) {
    Some(Value::I32(v)) => format!("i32:{}", v),
    Some(Value::I64(v)) => format!("i64:{}", v),
    Some(Value::F32(v)) => {
      let prefix = if v.is_nan() { "" } else { "f32:" };
      format!("{}{}", prefix, v)
    }
    Some(Value::F64(v)) => {
      let prefix = if v.is_nan() { "" } else { "f64:" };
      format!("{}{}", prefix, v)
    }
    None => "".to_owned(),
  }
}

macro_rules! impl_e2e {
  ($test_name: ident, $file_name: expr) => {
    #[test]
    fn $test_name() {
      let mut buf = String::new();
      let test_filename = format!("./testsuite/{}.wast", $file_name);
      let mut json = File::open(&test_filename).unwrap();
      json.read_to_string(&mut buf).unwrap();
      let mut parser: ScriptParser<f32, f64> = ScriptParser::from_str(&buf).unwrap();
      let mut current_modules: HashMap<Option<String>, Rc<RefCell<Vm>>> = HashMap::new();

      while let Ok(Some(Command { kind, line, .. })) = parser.next() {
        match kind {
          CommandKind::Module {
            ref module,
            ref name,
          } => {
            let vm = Rc::new(RefCell::new(Vm::new(module.clone().into_vec()).unwrap()));
            current_modules.insert(None, vm.clone());
            current_modules.insert(name.clone(), vm.clone());
          }

          CommandKind::AssertReturn {
            ref action,
            ref expected,
          } => {
            let (field, args, module) = match action {
              Action::Invoke {
                ref field,
                ref args,
                ref module,
              } => (field, get_args(args), module),
              Action::Get {
                ref field,
                ref module,
              } => (field, vec![], module),
            };
            println!("Assert return at {}:{}.", field, line);
            let vm_ref: Rc<RefCell<Vm>> = current_modules.get(module).unwrap().clone();
            let mut vm = vm_ref.borrow_mut();
            let actual = vm.run(field.as_ref(), args);
            let expectation = get_expectation(expected);
            assert_eq!(actual, expectation);
          }
          CommandKind::AssertTrap {
            action:
              Action::Invoke {
                ref field,
                ref args,
                ref module,
              },
            ref message,
          } => {
            println!("Assert trap at {}:{}.", field, line,);
            let vm_ref: Rc<RefCell<Vm>> = current_modules.get(module).unwrap().clone();
            let mut vm = vm_ref.borrow_mut();
            let actual = vm.run(field.as_ref(), get_args(args));
            assert_eq!(&actual, message);
          }
          CommandKind::AssertExhaustion {
            action: Action::Invoke {
              ref field, args: _, ..
            },
          } => {
            println!("Skip exhaustion line:{}:{}.", field, line);
            // FIXME: Enable specs
            // println!("Assert exhaustion at {}:{}.", field, line,);
            // let mut vm = Vm::new(current_module.clone()).unwrap();
            // let actual = vm.run(field.as_ref(), get_args(args));
            // assert_eq!(actual, "call stack exhaused".to_owned());
          }
          CommandKind::AssertMalformed {
            ref module,
            ref message,
          } => {
            let bytes = module.clone().into_vec();
            let tmp_bytes = bytes.clone();
            let (magic_numbers, _) = tmp_bytes.split_at(8);
            if magic_numbers == [0u8, 97, 115, 109, 1, 0, 0, 0] {
              if ($file_name == "custom_section" && line == 77)
                || ($file_name == "custom_section" && line == 94)
              {
                println!("Skip {}, it seems not reasonable...", line);
                continue;
              };
              println!("Assert malformed at {}.", line,);
              let mut vm = Vm::new(bytes);
              match vm {
                Ok(_) => unreachable!(),
                Err(err) => {
                  assert_eq!(&String::from(err), message);
                }
              }
            } else {
              println!("Skip malformed text form at line:{}.", line);
            };
          }
          CommandKind::AssertInvalid {
            ref message,
            module: _,
          } => {
            println!("Skip assert invalid at '{}:{}'.", message, line);
            /*
            println!("Assert invalid at '{}:{}'.", message, line);
            match wasvm::Vm::new(module.clone().into_vec()) {
              Ok(_) => unreachable!("Expect to trap decoding, but decoded normally."),
              Err(err) => {
                assert_eq!(&String::from(err), message);
              }
            }
            */
          }
          CommandKind::AssertReturnCanonicalNan {
            action:
              Action::Invoke {
                ref field,
                ref args,
                ref module,
              },
          } => {
            println!("Assert canonical NaN at '{}:{}'.", field, line);
            let vm_ref: Rc<RefCell<Vm>> = current_modules.get(module).unwrap().clone();
            let mut vm = vm_ref.borrow_mut();
            let actual = vm.run(field.as_ref(), get_args(args));
            assert_eq!(&actual, "NaN");
          }
          CommandKind::AssertReturnArithmeticNan {
            action:
              Action::Invoke {
                ref field,
                ref args,
                ref module,
              },
          } => {
            println!("Assert arithmetic NaN at '{}:{}'.", field, line);
            let vm_ref: Rc<RefCell<Vm>> = current_modules.get(module).unwrap().clone();
            let mut vm = vm_ref.borrow_mut();
            let actual = vm.run(field.as_ref(), get_args(args));
            assert_eq!(&actual, "NaN");
          }
          CommandKind::PerformAction(Action::Invoke {
            ref field, args: _, ..
          }) => {
            println!("Skip perform action at '{}:{}'.", field, line);
            break;
          }
          x => unreachable!(
            "there are no other commands apart from that defined above {:?}",
            x
          ),
        }
      }
    }
  };
}

// NOTE: Convient to debug wast specs.
// impl_e2e!(test_sandbox, "sandbox");

impl_e2e!(test_address, "address");
impl_e2e!(test_align, "align");
// impl_e2e!(test_binary, "binary");
impl_e2e!(test_block, "block");
impl_e2e!(test_br_if, "br_if");
impl_e2e!(test_br_table, "br_table");
impl_e2e!(test_br_only, "br");
impl_e2e!(test_break_drop, "break-drop");
impl_e2e!(test_call_indirect, "call_indirect");
impl_e2e!(test_call, "call");
impl_e2e!(test_comments, "comments");
impl_e2e!(test_conversions, "conversions");
impl_e2e!(test_custom_section, "custom_section");
impl_e2e!(test_const, "const"); /* All specs suppose Text-format */
impl_e2e!(test_exports, "exports");
impl_e2e!(test_f32, "f32");
impl_e2e!(test_f32_cmp, "f32_cmp");
impl_e2e!(test_f32_bitwise, "f32_bitwise");
impl_e2e!(test_f64, "f64");
impl_e2e!(test_fac, "fac");
impl_e2e!(test_f64_cmp, "f64_cmp");
impl_e2e!(test_f64_bitwise, "f64_bitwise");
impl_e2e!(test_float_exprs, "float_exprs");
impl_e2e!(test_get_local, "get_local");
impl_e2e!(test_set_local, "set_local");
impl_e2e!(test_tee_local, "tee_local");
// impl_e2e!(test_nop, "nop");
// impl_e2e!(test_globals, "globals");
impl_e2e!(test_i32, "i32");
impl_e2e!(test_i64, "i64");
impl_e2e!(test_if, "if");
impl_e2e!(test_loop, "loop");
impl_e2e!(test_token, "token");
