#![feature(test)]
extern crate test;
extern crate wasvm;

use std::fs;
use std::io::Read;
use wasvm::Vm;

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
fn bench_fib_recursive(b: &mut test::Bencher) {
  b.iter(|| {
    let mut file = fs::File::open("./tmp/fib_recursive.wasm").unwrap();
    let mut buffer = vec![];
    file.read_to_end(&mut buffer).unwrap();
    let mut vm = Vm::new(buffer).unwrap();
    assert_eq!(vm.run("app_main", vec![]), "i32:9227465");
  });
}
