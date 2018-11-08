use std::collections::HashMap;
mod byte;
mod utils;

#[derive(Debug, PartialEq)]
enum Op {
    Const(i32),
}

#[derive(Debug, PartialEq)]
struct FunctionType {
    parameters: Vec<byte::ValueTypes>,
    returns: Vec<byte::ValueTypes>,
}

#[derive(Debug, PartialEq)]
struct FunctionInstance {
    function_type: FunctionType,
    locals: Vec<byte::Values>,
    type_idex: u32,
    body: Vec<Op>,
}

#[derive(Debug, PartialEq)]
struct Store {
    function_instances: HashMap<String, FunctionInstance>,
}

impl Store {
    fn new(bytecode: Vec<byte::Code>) -> Self {
        //     SectionType,
        //         TypeFunction,
        //         ValueType(ValueTypes::I32),
        let _ = bytecode.get(0);
        let _ = bytecode.get(1);
        let _ = bytecode.get(2);

        let function_type = FunctionType {
            parameters: vec![],
            returns: vec![match bytecode.get(2) {
                Some(&byte::Code::ValueType(ref t)) => t.to_owned(),
                _ => unreachable!(),
            }],
        };

        //     SectionFunction,
        //         IdxOfType(0),
        let _ = bytecode.get(3);
        let _ = bytecode.get(4);
        let type_idex = match bytecode.get(4) {
            Some(&byte::Code::IdxOfType(n)) => n as u32,
            _ => unreachable!(),
        };

        //     SectionExport,
        //         ExportName("_subject".to_owned()),
        //         IdxOfFunction(0),
        let _ = bytecode.get(5);
        let _ = bytecode.get(6);
        let _ = bytecode.get(7);
        let key = match bytecode.get(6) {
            Some(&byte::Code::ExportName(ref name)) => name.to_owned(),
            _ => unreachable!(),
        };

        //     SectionCode,
        //         ConstI32,
        //         Value(Values::I32(42))
        let _ = bytecode.get(8);
        let _ = bytecode.get(9);
        let _ = bytecode.get(10);
        let body = vec![Op::Const(match bytecode.get(10) {
            Some(&byte::Code::Value(byte::Values::I32(n))) => n,
            _ => unreachable!(),
        })];

        let locals: Vec<byte::Values> = vec![];
        let mut function_instances = HashMap::new();
        function_instances.insert(
            key,
            FunctionInstance {
                function_type,
                type_idex,
                locals,
                body,
            },
        );
        Store { function_instances }
    }
}

#[derive(Debug)]
pub struct Vm {
    store: Store,
    stack: Vec<Op>,
}

impl Vm {
    pub fn new(bytes: Vec<u8>) -> Self {
        let mut bytes = byte::Byte::new(bytes);
        let _ = bytes.decode();
        Vm {
            store: Store::new(bytes.bytes_decoded),
            stack: vec![],
        }
    }

    pub fn run(&mut self) {
        // match self.stack.pop_front() {
        //     Some(v) => v,
        //     _ => Value::I32(0),
        // }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::FromIterator;
    use utils::read_wasm;

    #[test]
    fn it_can_organize_functions() {
        let wasm = read_wasm("./dist/constant.wasm").unwrap();
        let vm = Vm::new(wasm);
        assert_eq!(
            vm.store,
            Store {
                function_instances: HashMap::from_iter(
                    vec![(
                        "_subject".to_owned(),
                        FunctionInstance {
                            function_type: FunctionType {
                                parameters: vec![],
                                returns: vec![byte::ValueTypes::I32],
                            },
                            locals: vec![],
                            type_idex: 0,
                            body: vec![Op::Const(42)],
                        }
                    )].into_iter()
                )
            }
        );
    }

    //     #[test]
    //     fn it_can_push_constant() {
    //         let wasm = read_wasm("./dist/constant.wasm").unwrap();
    //         let mut vm = Vm::new(wasm);
    //         vm.decode();
    //         assert_eq!(vm.stack.pop_front(), Some(Value::I32(42)));
    //     }

    //     #[test]
    //     fn it_can_organize_modules() {
    //         let wasm = read_wasm("./dist/constant.wasm").unwrap();
    //         let mut vm = Vm::new(wasm);
    //         vm.decode();
    //         assert_eq!(
    //             vm.module,
    //             Module {
    //                 types: vec![(vec![], vec![ValueType::I32])],
    //                 func_addresses: vec![0]
    //             }
    //         );
    //     }

    //     #[test]
    //     fn it_can_evaluate_constant() {
    //         let wasm = read_wasm("./dist/constant.wasm").unwrap();
    //         let mut vm = Vm::new(wasm);
    //         vm.decode();
    //         assert_eq!(vm.run(), Value::I32(42));
    //     }
}
