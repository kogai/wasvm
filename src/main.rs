use std::collections::VecDeque;
use std::fs;
use std::io;
use std::io::Read;

#[derive(Debug, PartialEq)]
enum Value {
    I32(i32),
}

#[derive(Debug)]
struct Vm {
    bytes: Vec<u8>,
    len: usize,
    bp: usize,
    stack: VecDeque<Value>,
}

impl Vm {
    fn new(bytes: Vec<u8>) -> Self {
        Vm {
            len: bytes.len(),
            bytes,
            bp: 0,
            stack: VecDeque::new(),
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
                        let &_result_type = self.next().unwrap();
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
                        let &v = self.next().unwrap();
                        self.stack.push_front(Value::I32(v as i32))
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_push_constant() {
        let mut file = fs::File::open("./dist/constant.wasm").unwrap();
        let mut tmp = [0; 8];
        let mut buffer = vec![];
        let _ = file.read_exact(&mut tmp).unwrap();
        file.read_to_end(&mut buffer).unwrap();

        let mut vm = Vm::new(buffer);
        while vm.has_next() {
            vm.decode_section();
        }
        assert_eq!(vm.stack.pop_front(), Some(Value::I32(42)));
    }
}

fn main() -> io::Result<()> {
    let mut file = fs::File::open("./dist/constant.wasm")?;
    let mut tmp = [0; 4];
    let _drop_magic_number = file.read_exact(&mut tmp)?;
    let _drop_version = file.read_exact(&mut tmp)?;

    let mut buffer = vec![];
    file.read_to_end(&mut buffer)?;
    buffer.reverse();

    let mut wasm_bytes = Vm::new(buffer);
    while wasm_bytes.has_next() {
        wasm_bytes.decode_section();
    }
    Ok(())
}
