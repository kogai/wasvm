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

impl ValueType {
    fn from_byte(code: u8) -> Self {
        match code {
            0x7f => ValueType::I32,
            _ => unreachable!(),
        }
    }
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
    bytes: Vec<u8>,
    len: usize,
    bp: usize,
    store: Store,
    stack: VecDeque<Value>,
    module: Module,
}

impl Vm {
    pub fn new(bytes: Vec<u8>) -> Self {
        Vm {
            len: bytes.len(),
            bytes,
            bp: 0,
            store: Store::new(),
            stack: VecDeque::new(),
            module: Module {
                types: vec![],
                func_addresses: vec![],
            },
        }
    }

    fn has_next(&self) -> bool {
        self.bp < self.len
    }

    fn next(&mut self) -> Option<&u8> {
        let el = self.bytes.get(self.bp);
        self.bp += 1;
        el
    }

    fn decode_section(&mut self) {
        match self.next() {
            Some(0x1) => {
                // Type Section
                let &size_of_section = self.next().unwrap();
                let &_num_of_type = self.next().unwrap();
                // FIXME: Should iterate over num_of_type
                match self.next() {
                    Some(0x60) => {
                        let &_num_of_param = self.next().unwrap();
                        let &_num_of_result = self.next().unwrap();
                        let &result_type = self.next().unwrap();
                        self.module
                            .types
                            .push((vec![], vec![ValueType::from_byte(result_type)]));
                    }
                    _ => {}
                }
                println!("Hit type section, consist of {:?} bytes", &size_of_section);
            }
            Some(0x3) => {
                // Function section
                let &size_of_section = self.next().unwrap();
                // FIXME: Should iterate over num_of_type_idx
                let &_num_of_type_idx = self.next().unwrap();
                let &_type_idx = self.next().unwrap();
                println!(
                    "Hit function section, consist of {:?} bytes",
                    &size_of_section
                );
            }
            Some(0x7) => {
                // Export section
                let &size_of_section = self.next().unwrap();
                // FIXME: Should iterate over num_of_export
                let &_num_of_export = self.next().unwrap();
                let &_num_of_name = self.next().unwrap();
                let mut buf = vec![];
                for _ in 0.._num_of_name {
                    let &el = self.next().unwrap();
                    buf.push(el);
                }

                let &_export_description = self.next().unwrap(); // == 0x0
                let &_function_idx = self.next().unwrap();
                println!(
                    "Hit export section, consist of {:?} bytes. Function named {:?}",
                    &size_of_section,
                    String::from_utf8(buf).unwrap()
                );
            }
            Some(0xa) => {
                // Code section
                let &size_of_section = self.next().unwrap();
                // FIXME: Should iterate over num_of_code
                let &_num_of_code = self.next().unwrap();
                let &_size_of_function = self.next().unwrap();
                let &_num_of_param = self.next().unwrap();
                match self.next() {
                    Some(0x41) => {
                        // FIXME: Decode expressions properly.
                        // This implementation may don't make any sense.
                        let &v = self.next().unwrap();
                        let idx = self.module.func_addresses.len();
                        self.stack.push_front(Value::I32(v as i32));
                        self.module.func_addresses.push(idx as i32);
                        let body = vec![Op::Const(v as i32)];
                        self.store.function_instance.push(FunctionInstance { body });
                    }
                    Some(_) | None => unimplemented!(),
                }
                let &_end = self.next().unwrap();
                println!("Hit code section, consist of {:?} bytes.", &size_of_section);
            }
            Some(_) => unimplemented!(),
            None => {}
        }
    }

    pub fn decode(&mut self) {
        while self.has_next() {
            self.decode_section();
        }
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
