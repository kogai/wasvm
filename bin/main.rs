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

#[derive(Clone)]
struct Sample(usize, usize);

// NOTE: Compare which one is fast vec.push(el) or vec[idx] = el;
// And below is the result.
// test bench_assign ... bench:          59 ns/iter (+/- 1)
// test bench_push   ... bench:         624 ns/iter (+/- 5)

#[bench]
fn bench_assign(b: &mut test::Bencher) {
  let mut buf: Vec<Sample> = vec![Sample(0, 0); 100];
  b.iter(|| {
    for i in 0..100 {
      buf[i] = Sample(i, i);
    }
  });
}

#[bench]
fn bench_push(b: &mut test::Bencher) {
  let mut buf = Vec::with_capacity(100);
  b.iter(|| {
    for i in 0..100 {
      buf.push(Sample(i, i));
    }
  });
}

#[bench]
fn bench_fib(b: &mut test::Bencher) {
  b.iter(|| {
    let mut file = fs::File::open("./dist/fib.wasm").unwrap();
    let mut buffer = vec![];
    file.read_to_end(&mut buffer).unwrap();
    let mut vm = Vm::new(buffer).unwrap();
    // assert_eq!(vm.run("_subject", vec![Values::I32(35)]), "i32:9227465");
    assert_eq!(vm.run("_subject", vec![Values::I32(10)]), "i32:55");
  });
}
