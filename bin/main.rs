extern crate wasvm;

use std::env::args;
use std::fs;
use std::io;
use std::io::Read;
use wasvm::byte;

fn main() -> io::Result<()> {
  let arguments = args().collect::<Vec<String>>();
  let (_, arguments) = arguments.split_at(1);
  match arguments.split_first() {
    Some((file_name, arguments)) => {
      let mut file = fs::File::open(format!("./dist/{}.wasm", file_name))?;
      let mut tmp = [0; 4];
      let _drop_magic_number = file.read_exact(&mut tmp)?;
      let _drop_version = file.read_exact(&mut tmp)?;

      let mut buffer = vec![];
      file.read_to_end(&mut buffer)?;

      let mut vm = wasvm::Vm::new(buffer);
      vm.run(
        arguments
          .iter()
          .map(|v| i32::from_str_radix(v, 10).expect("Parameters must be i32"))
          .map(|v| byte::Values::I32(v))
          .collect::<Vec<byte::Values>>(),
      );
      println!("{:?}", vm.stack.pop().unwrap());
    }
    _ => unreachable!("Should specify file-name"),
  };
  Ok(())
}
