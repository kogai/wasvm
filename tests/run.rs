#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[cfg(test)]
extern crate wasvm;
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
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestCases {
  source_filename: String,
  commands: Vec<TestCase>,
}

macro_rules! impl_e2e {
  ($test_name: ident, $file_name: expr) => {
    #[test]
    fn $test_name() {
      let mut buffer_json = vec![];
      let mut json = File::open(format!("dist/{}.json", $file_name)).unwrap();
      json.read_to_end(&mut buffer_json).unwrap();
      let test_cases = serde_json::from_slice::<TestCases>(&buffer_json).unwrap();
      let (module, assertions) = test_cases.commands.split_first().unwrap();
      let wasm_file = if let TestCase::Module { line: _, filename } = module {
        let mut file = File::open(format!("dist/{}", filename)).unwrap();
        let mut tmp = [0; 8];
        let mut buffer = vec![];
        let _ = file.read_exact(&mut tmp).unwrap();
        file.read_to_end(&mut buffer).unwrap();
        buffer
      } else {
        unreachable!();
      };

      for assertion in assertions {
        match assertion {
          TestCase::AssertReturn {
            line,
            action: Action::Invoke { field, args },
            expected,
          } => {
            // if *line != 39 {
            //   continue;
            // };
            println!("Testing spec at line:{}.", line);
            let mut vm = wasvm::Vm::new(wasm_file.clone());
            vm.run(
              field,
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
                    x => unimplemented!("{:?} is not implemented yet", x),
                  }
                }).collect::<Vec<Values>>(),
            );
            let exp = expected.get(0).unwrap().to_owned();
            let expectation = match (exp.value_type.as_ref(), exp.value) {
              ("i32", Some(value)) => {
                let actual_value = value.parse::<u32>().unwrap() as i32;
                Some(format!("{}", actual_value))
              }
              (_, None) => None,
              _ => unimplemented!(),
            };
            assert_eq!(vm.get_result(), expectation);
          }
          // TODO: Test AssertTrap
          _ => {
            println!("Skip assert_trap");
          }
        }
      }
    }
  };
}

impl_e2e!(test_i32, "i32");
// impl_e2e!(test_i64, "i64");
