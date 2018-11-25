extern crate wabt;

#[cfg(test)]
extern crate wasvm;
use std::fs::File;
use std::io::Read;
use wabt::script::{Action, Command, CommandKind, ModuleBinary, ScriptParser, Value};
use wasvm::value::Values;

fn get_args(args: &Vec<Value<f32, f64>>) -> Vec<Values> {
  args
    .iter()
    .map(|v| match v {
      Value::I32(value) => Values::I32(*value),
      Value::I64(value) => Values::I64(*value),
      Value::F32(value) => Values::F32(*value),
      _ => unimplemented!(),
    })
    .collect()
}

fn get_expectation(expected: &Vec<Value>) -> String {
  let v = expected.get(0).unwrap().to_owned();
  match v {
    Value::I32(value) => format!("i32:{}", value),
    Value::I64(value) => format!("i64:{}", value),
    Value::F32(value) => format!("f32:{}", value),
    Value::F64(value) => format!("f64:{}", value),
  }
}

fn do_test(parser: &mut ScriptParser, module: &ModuleBinary) {
  loop {
    match parser.next().unwrap() {
      Some(Command {
        line,
        kind:
          CommandKind::AssertReturn {
            action:
              Action::Invoke {
                ref field,
                ref args,
                ..
              },
            ref expected,
          },
      }) => {
        if line != 47 {
          continue;
        };
        println!("Assert return at line:{}.", line);
        let mut vm = wasvm::Vm::new(module.clone().into_vec()).unwrap();
        let actual = vm.run(field.as_ref(), get_args(args));
        let expectation = get_expectation(expected);
        assert_eq!(actual, expectation);
      }
      Some(Command {
        line,
        kind:
          CommandKind::AssertTrap {
            action:
              Action::Invoke {
                ref field,
                ref args,
                ..
              },
            ref message,
          },
      }) => {
        println!("Assert trap at line:{}.", line,);
        let mut vm = wasvm::Vm::new(module.clone().into_vec()).unwrap();
        let actual = vm.run(field.as_ref(), get_args(args));
        assert_eq!(&actual, message);
      }
      Some(Command {
        line,
        kind: CommandKind::AssertMalformed {
          module: _,
          message: _,
        },
      }) => {
        println!("Skip malformed at line:{}.", line);
      }
      Some(Command {
        line,
        kind:
          CommandKind::AssertReturnCanonicalNan {
            action:
              Action::Invoke {
                ref field,
                ref args,
                ..
              },
          },
      }) => {
        println!("Assert canonical NaN at line:{}.", line);
        let mut vm = wasvm::Vm::new(module.clone().into_vec()).unwrap();
        let actual = vm.run(field.as_ref(), get_args(args));
        assert_eq!(&actual, "f32:NaN");
      }
      Some(Command {
        line,
        kind: CommandKind::AssertReturnArithmeticNan { action },
      }) => {
        println!("Skip arithmetic NaN at line:{}.", line);
      }
      Some(Command {
        kind: CommandKind::Module { ref module, .. },
        ..
      }) => do_test(parser, module),
      None => break,
      x => unimplemented!("{:?}", x),
    }
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

      while let Ok(Some(Command { kind, .. })) = parser.next() {
        match kind {
          CommandKind::Module { ref module, .. } => do_test(&mut parser, module),
          x => unreachable!(
            "there are no other commands apart from that defined above {:?}",
            x
          ),
        }
      }
    }
  };
}

impl_e2e!(test_i32, "i32");
impl_e2e!(test_i64, "i64");
impl_e2e!(test_address, "address");
impl_e2e!(test_f32, "f32");
