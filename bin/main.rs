extern crate wasvm;

use std::fs;
use std::io;
use std::io::Read;

fn main() -> io::Result<()> {
  let mut file = fs::File::open("./dist/constant.wasm")?;
  let mut tmp = [0; 4];
  let _drop_magic_number = file.read_exact(&mut tmp)?;
  let _drop_version = file.read_exact(&mut tmp)?;

  let mut buffer = vec![];
  file.read_to_end(&mut buffer)?;

  let mut vm = wasvm::Vm::new(buffer);
  vm.run(vec![]);
  Ok(())
}
