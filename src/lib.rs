use std::collections::VecDeque;
mod byte;

type FnType = (Vec<ValueType>, Vec<ValueType>);

#[derive(Debug, PartialEq)]
pub enum Value {
    I32(i32),
}

#[derive(Debug, PartialEq)]
enum Op {
    Const(i32),
}

#[derive(Debug, PartialEq)]
struct FunctionInstance {
    body: Vec<Op>,
}

#[derive(Debug, PartialEq)]
enum ValueType {
    I32,
    // I64,
    // F32,
    // F63,
}

#[derive(Debug, PartialEq)]
struct Module {
    types: Vec<FnType>,
    func_addresses: Vec<i32>,
}

#[derive(Debug, PartialEq)]
struct Store {
    function_instance: Vec<FunctionInstance>,
}

impl Store {
    fn new() -> Self {
        Store {
            function_instance: vec![],
        }
    }
}

#[derive(Debug)]
pub struct Vm {
    bytes: byte::Byte,
    bp: usize,
    store: Store,
    stack: VecDeque<Value>,
    module: Module,
}

impl Vm {
    pub fn new(bytes: Vec<u8>) -> Self {
        Vm {
            bytes: byte::Byte::new(bytes),
            bp: 0,
            store: Store::new(),
            stack: VecDeque::new(),
            module: Module {
                types: vec![],
                func_addresses: vec![],
            },
        }
    }

    pub fn decode(&mut self) {
        let _ = self.bytes.decode();
    }

    pub fn run(&mut self) -> Value {
        match self.stack.pop_front() {
            Some(v) => v,
            _ => Value::I32(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io;
    use std::io::Read;
    use std::path::Path;

    fn read_wasm<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
        let mut file = fs::File::open(path)?;
        let mut tmp = [0; 8];
        let mut buffer = vec![];
        let _ = file.read_exact(&mut tmp)?;
        file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    #[test]
    fn it_can_push_constant() {
        let wasm = read_wasm("./dist/constant.wasm").unwrap();
        let mut vm = Vm::new(wasm);
        vm.decode();
        assert_eq!(vm.stack.pop_front(), Some(Value::I32(42)));
    }

    #[test]
    fn it_can_organize_modules() {
        let wasm = read_wasm("./dist/constant.wasm").unwrap();
        let mut vm = Vm::new(wasm);
        vm.decode();
        assert_eq!(
            vm.module,
            Module {
                types: vec![(vec![], vec![ValueType::I32])],
                func_addresses: vec![0]
            }
        );
    }

    #[test]
    fn it_can_evaluate_constant() {
        let wasm = read_wasm("./dist/constant.wasm").unwrap();
        let mut vm = Vm::new(wasm);
        vm.decode();
        assert_eq!(vm.run(), Value::I32(42));
    }

    #[test]
    fn it_can_organize_functions() {
        let wasm = read_wasm("./dist/constant.wasm").unwrap();
        let mut vm = Vm::new(wasm);
        vm.decode();
        assert_eq!(
            vm.store,
            Store {
                function_instance: vec![FunctionInstance {
                    body: vec![Op::Const(42)],
                }]
            }
        );
    }

}
