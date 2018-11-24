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
  #[serde(rename = "assert_return_canonical_nan")]
  AssertReturnCanonicalNan {
    line: usize,
    action: Action,
    expected: Vec<TypeValue>,
  },
  #[serde(rename = "assert_return_arithmetic_nan")]
  AssertReturnArithmeticNan {
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

fn from_string(value_type: String, value: Option<String>) -> String {
  match (value_type.as_ref(), value) {
    ("i32", Some(value)) => {
      let actual_value = value.parse::<u32>().unwrap() as i32;
      format!("{}", actual_value)
    }
    ("i64", Some(value)) => {
      let actual_value = value.parse::<u64>().unwrap() as i64;
      format!("{}", actual_value)
    }
    ("f32", Some(value)) => {
      let actual_value = f32::from_bits(value.parse::<u32>().unwrap());
      format!("{}", actual_value)
    }
    (_, None) => "".to_owned(),
    _ => unimplemented!(),
  }
}

fn get_args(args: &Vec<TypeValue>) -> Vec<Values> {
  args
    .iter()
    .map(|v| match (v.value_type.as_ref(), v.value.clone()) {
      ("i32", Some(value)) => {
        let actual_value = value.parse::<u32>().unwrap() as i32;
        Values::I32(actual_value)
      }
      ("i64", Some(value)) => {
        let actual_value = value.parse::<u64>().unwrap() as i64;
        Values::I64(actual_value)
      }
      ("f32", Some(value)) => {
        let actual_value = f32::from_bits(value.parse::<u32>().unwrap());
        Values::F32(actual_value)
      }
      _ => unimplemented!(),
    })
    .collect()
}

fn get_expectation(expected: &Vec<TypeValue>) -> String {
  let v = expected.get(0).unwrap().to_owned();
  match (v.value_type.as_ref(), v.value) {
    ("i32", Some(value)) => {
      let actual_value = value.parse::<u32>().unwrap() as i32;
      format!("{}", actual_value)
    }
    ("i64", Some(value)) => {
      let actual_value = value.parse::<u64>().unwrap() as i64;
      format!("{}", actual_value)
    }
    ("f32", Some(value)) => {
      let actual_value = f32::from_bits(value.parse::<u32>().unwrap());
      format!("{}", actual_value)
    }
    (_, None) => "".to_owned(),
    _ => unimplemented!(),
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
            println!("## Testing specs [{}].", filename);
            // FIXME: Skip specs using floating-point until implemented.
            if (&filename == "address.3.wasm") | (&filename == "address.4.wasm") {
              break;
            }

            let mut file = File::open(format!("dist/{}", filename)).unwrap();
            let mut wasm_exec = vec![];
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
                  // if line != 60 {
                  //   continue;
                  // };
                  println!("Testing spec at line:{}.", line);
                  let actual = vm.run(field.as_ref(), get_args(args));
                  let expectation = get_expectation(expected);
                  assert_eq!(actual, expectation);
                }
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
impl_e2e!(test_f32, "f32");
impl_e2e!(test_address, "address");
