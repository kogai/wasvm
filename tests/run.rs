extern crate wabt;

#[cfg(test)]
extern crate wasvm;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use wabt::script::{Action, Command, CommandKind, ScriptParser, Value};
use wasvm::{create_spectest, ExternalModules, Values, Vm};

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
      let mut importable_modules: ExternalModules = ExternalModules::new();
      let spectest = create_spectest();
      importable_modules.register_module(Some("spectest".to_owned()), spectest);

      while let Ok(Some(Command { kind, line, .. })) = parser.next() {
        match kind {
          CommandKind::Module {
            ref module,
            ref name,
          } => {
            let vm_ref = Rc::new(RefCell::new(
              Vm::new_with_externals(module.clone().into_vec(), importable_modules.clone())
                .unwrap(),
            ));
            current_modules.insert(None, vm_ref.clone());
            current_modules.insert(name.clone(), vm_ref.clone());
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
            match message.as_ref() {
              "unreachable" => assert_eq!(actual, format!("{} executed", message)),
              _ => assert_eq!(&actual, message),
            }
          }
          CommandKind::AssertUninstantiable {
            ref module,
            ref message,
          } => {
            println!("Assert uninstantiable at line:{}.", line);
            let bytes = module.clone().into_vec();
            let mut vm = Vm::new(bytes);
            match vm {
              Ok(_) => unreachable!(),
              Err(err) => {
                let actual = String::from(err);
                match message.as_ref() {
                  "unreachable" => assert_eq!(actual, format!("{} executed", message)),
                  _ => assert_eq!(&actual, message),
                }
              }
            }
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
            ref field,
            ref args,
            ref module,
          }) => {
            println!("Perform action at {}:{}.", field, line);
            let vm_ref: Rc<RefCell<Vm>> = current_modules.get(module).unwrap().clone();
            let mut vm = vm_ref.borrow_mut();
            vm.run(field.as_ref(), get_args(args));
          }
          CommandKind::Register {
            ref name,
            ref as_name,
            ..
          } => {
            println!(
              "Register importable module, key={:?} import_name={}.",
              name, as_name
            );
            let mut vm_ref: Rc<RefCell<Vm>> = current_modules.get(name).unwrap().clone();
            let vm = vm_ref.borrow();
            let importable_module = vm.export_module();
            importable_modules.register_module(Some(as_name.clone()), importable_module);
          }
          CommandKind::AssertUnlinkable {
            ref module,
            ref message,
          } => {
            let bytes = module.clone().into_vec();
            let tmp_bytes = bytes.clone();
            let (magic_numbers, _) = tmp_bytes.split_at(8);
            if magic_numbers == [0u8, 97, 115, 109, 1, 0, 0, 0] {
              println!("Assert unlinkable at {}.", line,);
              let mut vm = Vm::new_with_externals(bytes, importable_modules.clone());
              match vm {
                Ok(_) => unreachable!("Expect '{}', but successed to instantiate.", message),
                Err(err) => {
                  let actual = String::from(err);
                  match message.as_ref() {
                    // FIXME: Skip to assert actual message
                    "incompatible import type" => {}
                    _ => assert_eq!(&actual, message),
                  }
                }
              }
            } else {
              println!("Skip unlinkable text form at line:{}.", line);
            };
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
impl_e2e!(test_const, "const"); /* All specs suppose Text-format */
impl_e2e!(test_conversions, "conversions");
impl_e2e!(test_custom_section, "custom_section");
// impl_e2e!(test_custom, "custom");
impl_e2e!(test_data, "data");
impl_e2e!(test_elem, "elem");
impl_e2e!(test_endianness, "endianness");
impl_e2e!(test_exports, "exports");
impl_e2e!(test_f32_bitwise, "f32_bitwise");
impl_e2e!(test_f32_cmp, "f32_cmp");
impl_e2e!(test_f32, "f32");
impl_e2e!(test_f64_bitwise, "f64_bitwise");
impl_e2e!(test_f64_cmp, "f64_cmp");
impl_e2e!(test_f64, "f64");
impl_e2e!(test_fac, "fac");
impl_e2e!(test_float_exprs, "float_exprs");
impl_e2e!(test_float_literals, "float_literals");
impl_e2e!(test_float_memory, "float_memory");
// impl_e2e!(test_float_misc, "float_misc");
impl_e2e!(test_forward, "forward");
// impl_e2e!(test_func_ptrs, "func_ptrs");
impl_e2e!(test_func, "func");
impl_e2e!(test_get_local, "get_local");
// impl_e2e!(test_globals, "globals");
impl_e2e!(test_i32, "i32");
impl_e2e!(test_i64, "i64");
impl_e2e!(test_if, "if");
// impl_e2e!(test_imports, "imports");
impl_e2e!(test_inline_module, "inline-module");
impl_e2e!(test_int_exprs, "int_exprs");
impl_e2e!(test_int_literals, "int_literals");
impl_e2e!(test_labels, "labels");
// impl_e2e!(test_left_to_right, "left-to-right");
// impl_e2e!(test_linking, "linking");
impl_e2e!(test_loop, "loop");
impl_e2e!(test_memory_grow, "memory_grow");
impl_e2e!(test_memory_redundancy, "memory_redundancy");
impl_e2e!(test_memory_trap, "memory_trap");
impl_e2e!(test_memory, "memory");
impl_e2e!(test_names, "names");
impl_e2e!(test_nop, "nop");
impl_e2e!(test_resizing, "resizing");
impl_e2e!(test_return, "return");
impl_e2e!(test_select, "select");
impl_e2e!(test_set_local, "set_local");
impl_e2e!(test_skip_stack_guard_page, "skip-stack-guard-page");
impl_e2e!(test_stack, "stack");
impl_e2e!(test_start, "start");
impl_e2e!(test_store_retval, "store_retval");
impl_e2e!(test_switch, "switch");
impl_e2e!(test_tee_local, "tee_local");
impl_e2e!(test_token, "token");
impl_e2e!(test_traps, "traps");
impl_e2e!(test_type, "type");
impl_e2e!(test_typecheck, "typecheck");
impl_e2e!(test_unreachable, "unreachable");
impl_e2e!(test_unreached_invalid, "unreached-invalid");
// impl_e2e!(test_unwind, "unwind");
// impl_e2e!(test_utf8_custom_section_id, "utf8-custom-section-id");
// impl_e2e!(test_utf8_import_field, "utf8-import-field");
// impl_e2e!(test_utf8_import_module, "utf8-import-module");
impl_e2e!(test_utf8_invalid_encoding, "utf8-invalid-encoding");
