#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[cfg(test)]
extern crate wasvm;
use std::collections::LinkedList;
use std::fs::File;
use std::io::Read;
use wasvm::value::Values;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct TypeValue {
  #[serde(rename = "type")]
  value_type: String,
  value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
enum Action {
  #[serde(rename = "invoke")]
  Invoke { field: String, args: Vec<TypeValue> },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
enum TestCase {
  #[serde(rename = "module")]
  Module { line: usize, filename: String },
  #[serde(rename = "assert_return")]
  AssertReturn {
    line: usize,
    action: Action,
    expected: Vec<TypeValue>,
  },
  #[serde(rename = "assert_trap")]
  AssertTrap {
    line: usize,
    action: Action,
    text: String,
    expected: Vec<TypeValue>,
  },
  #[serde(rename = "assert_malformed")]
  AssertMalformed {
    line: usize,
    filename: String,
    text: String,
    module_type: String,
  },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestCases {
  source_filename: String,
  commands: LinkedList<TestCase>,
}

fn is_module_type(x: Option<&TestCase>) -> bool {
  match x {
    Some(TestCase::Module { .. }) | Some(TestCase::AssertMalformed { .. }) => true,
    _ => false,
  }
}

macro_rules! impl_e2e {
  ($test_name: ident, $file_name: expr) => {
    #[test]
    fn $test_name() {
      let mut buffer_json = vec![];
      let mut json = File::open(format!("dist/{}.json", $file_name)).unwrap();
      json.read_to_end(&mut buffer_json).unwrap();

      let test_cases = serde_json::from_slice::<TestCases>(&buffer_json).unwrap();
      let mut test_cases = test_cases.commands;
      while !test_cases.is_empty() {
        match test_cases.pop_front().unwrap() {
          TestCase::Module { line: _, filename } => {
            let mut file = File::open(format!("dist/{}", filename)).unwrap();
            let mut tmp = [0; 8];
            let mut wasm_exec = vec![];
            let _ = file.read_exact(&mut tmp).unwrap();
            file.read_to_end(&mut wasm_exec).unwrap();
            while !is_module_type(test_cases.front()) {
              match (test_cases.pop_front(), wasvm::Vm::new(wasm_exec.clone())) {
                (
                  Some(TestCase::AssertReturn {
                    line,
                    action:
                      Action::Invoke {
                        ref field,
                        ref args,
                      },
                    ref expected,
                  }),
                  Ok(ref mut vm),
                ) => {
                  if line != 60 {
                    continue;
                  };
                  println!("Testing spec at line:{}.", line);
                  let actual = vm.run(
                    field.as_ref(),
                    args
                      .iter()
                      .map(|v| {
                        let value = v.value.to_owned().unwrap();
                        let value_type = v.value_type.to_owned();
                        match value_type.as_ref() {
                          "i32" => {
                            let actual_value = value.parse::<u32>().unwrap() as i32;
                            Values::I32(actual_value)
                          }
                          "i64" => {
                            let actual_value = value.parse::<u64>().unwrap() as i64;
                            Values::I64(actual_value)
                          }
                          x => unimplemented!("{:?} is not implemented yet", x),
                        }
                      })
                      .collect::<Vec<Values>>(),
                  );
                  let exp = expected.get(0).unwrap().to_owned();
                  let expectation = match (exp.value_type.as_ref(), exp.value) {
                    ("i32", Some(value)) => {
                      let actual_value = value.parse::<u32>().unwrap() as i32;
                      format!("{}", actual_value)
                    }
                    ("i64", Some(value)) => {
                      let actual_value = value.parse::<u64>().unwrap() as i64;
                      format!("{}", actual_value)
                    }
                    (_, None) => "".to_owned(),
                    _ => unimplemented!(),
                  };
                  assert_eq!(actual, expectation);
                }
                // (Some(TestCase::AssertReturn { .. }), Err(_))
                // | (Some(TestCase::AssertTrap { .. }), Err(_))
                (Some(TestCase::AssertTrap { line, .. }), Err(_))
                | (Some(TestCase::AssertTrap { line, .. }), Ok(_)) => {
                  println!("Skip assert trap {}", line);
                }
                (None, _) => {
                  break;
                }
                (x, _) => unreachable!("{:?}", x),
              }
            }
          }
          TestCase::AssertMalformed { .. } => {
            continue;
          }
          _ => unreachable!(),
        }
      }
    }
  };
}

impl_e2e!(test_i32, "i32");
impl_e2e!(test_i64, "i64");
impl_e2e!(test_address, "address");
