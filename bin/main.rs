#![feature(test)]
extern crate test;
extern crate wasvm;

use std::env::args;
use std::fs;
use std::io;
use std::io::Read;
use wasvm::{Values, Vm};

fn main() -> io::Result<()> {
  let arguments = args().collect::<Vec<String>>();
  let (_, arguments) = arguments.split_at(1);
  match arguments.split_first() {
    Some((file_name, arguments)) => {
      let mut file = fs::File::open(format!("./{}.wasm", file_name))?;
      let mut buffer = vec![];
      file.read_to_end(&mut buffer)?;

      let mut vm = Vm::new(buffer).unwrap();
      let result = vm.run(
        "_subject",
        arguments
          .iter()
          .map(|v| i32::from_str_radix(v, 10).expect("Parameters must be i32"))
          .map(|v| Values::I32(v))
          .collect::<Vec<Values>>(),
      );
      println!("{:?}", result);
    }
    _ => unreachable!("Should specify file-name"),
  };
  Ok(())
}

pub fn add_two(x: usize) -> usize {
  x + 2
}

#[cfg(test)]
mod tests {
  use super::*;
  use test::Bencher;

  #[bench]
  fn bench_fib(b: &mut Bencher) {
    b.iter(|| {
      let mut file = fs::File::open("./dist/fib.wasm").unwrap();
      let mut buffer = vec![];
      file.read_to_end(&mut buffer).unwrap();
      let mut vm = Vm::new(buffer).unwrap();
      // assert_eq!(vm.run("_subject", vec![Values::I32(35)]), "i32:9227465");
      assert_eq!(vm.run("_subject", vec![Values::I32(10)]), "i32:55");
    });
  }
}
