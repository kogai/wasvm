use std::collections::HashMap;
mod byte;
mod utils;

#[derive(Debug, PartialEq, Clone)]
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
    fn new_function_instances(bytecode: &Vec<byte::Code>) -> HashMap<String, FunctionInstance> {
        let mut bytecode_cursor = 0;
        let locals = vec![];
        let mut body = vec![];
        let mut function_type = FunctionType {
            parameters: vec![],
            returns: vec![],
        };
        let mut type_idex = 0;
        let mut key = String::new();
        let mut function_instance = HashMap::new();

        while match bytecode.get(bytecode_cursor) {
            Some(byte::Code::SectionCode)
            | Some(byte::Code::SectionExport)
            | Some(byte::Code::SectionFunction)
            | Some(byte::Code::SectionType) => true,
            _ => false,
        } {
            match bytecode.get(bytecode_cursor) {
                Some(byte::Code::SectionCode) => {
                    bytecode_cursor += 1;
                    match bytecode.get(bytecode_cursor) {
                        Some(byte::Code::ConstI32) => {
                            bytecode_cursor += 1;
                            body.push(Op::Const(match bytecode.get(bytecode_cursor) {
                                Some(&byte::Code::Value(byte::Values::I32(n))) => n,
                                _ => unreachable!(),
                            }));
                            bytecode_cursor += 1;
                        }
                        _ => unreachable!(),
                    }
                }
                Some(byte::Code::SectionExport) => {
                    bytecode_cursor += 1;
                    key = match bytecode.get(bytecode_cursor) {
                        Some(&byte::Code::ExportName(ref name)) => name.to_owned(),
                        _ => unreachable!(),
                    };
                    bytecode_cursor += 1;
                    bytecode_cursor += 1;
                }
                Some(byte::Code::SectionFunction) => {
                    bytecode_cursor += 1;
                    type_idex = match bytecode.get(bytecode_cursor) {
                        Some(&byte::Code::IdxOfType(n)) => n as u32,
                        _ => unreachable!(),
                    };
                    bytecode_cursor += 1;
                }
                Some(byte::Code::SectionType) => {
                    bytecode_cursor += 1;
                    if let Some(byte::Code::TypeFunction) = bytecode.get(bytecode_cursor) {
                        bytecode_cursor += 1;
                        function_type
                            .returns
                            .push(match bytecode.get(bytecode_cursor) {
                                Some(byte::Code::ValueType(ref t)) => {
                                    bytecode_cursor += 1;
                                    t.to_owned()
                                }
                                _ => unreachable!(),
                            });
                    };
                }
                _ => unreachable!(),
            }
        }
        function_instance.insert(
            key,
            FunctionInstance {
                function_type,
                type_idex,
                locals,
                body,
            },
        );
        function_instance
    }

    fn new(bytecode: Vec<byte::Code>) -> Self {
        Store {
            function_instances: Store::new_function_instances(&bytecode),
        }
    }

    fn call(&self, key: &str) -> Option<&Vec<Op>> {
        self.function_instances.get(key).map(|f| &f.body)
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
        match self.store.call("_subject") {
            Some(expressions) => {
                for expression in expressions.iter() {
                    self.stack.push(expression.to_owned());
                }
            }
            None => println!("'_subject' did not implemented."),
        }
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

    #[test]
    fn it_can_evaluate_constant() {
        let wasm = read_wasm("./dist/constant.wasm").unwrap();
        let mut vm = Vm::new(wasm);
        vm.run();
        assert_eq!(vm.stack.pop(), Some(Op::Const(42)));
    }
}
