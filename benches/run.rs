#![allow(clippy::needless_range_loop)]
#![feature(test)]
extern crate flame;
extern crate test;
extern crate wasvm;

use std::fs;
use std::io::Read;
use wasvm::{decode_module, init_store, instantiate_module, Values};

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

macro_rules! impl_benches {
  ($test_name: ident, $bench_name: expr, $expect: expr) => {
    #[bench]
    fn $test_name(_b: &mut test::Bencher) {
      let mut file = fs::File::open(format!("./tmp/{}.wasm", $bench_name)).unwrap();
      let mut bytes = vec![];
      file.read_to_end(&mut bytes).unwrap();
      flame::start($bench_name);
      // b.iter(|| {
      let store = init_store();
      let module = decode_module(&bytes);
      let mut vm = instantiate_module(store, module, Default::default(), 65536).unwrap();
      assert_eq!(vm.run("app_main", vec![]).unwrap(), $expect);
      // });
      flame::end($bench_name);
      flame::dump_stdout();
    }
  };
}

impl_benches!(bench_fib_recursive, "fib_recursive", Values::I32(922_7465));
impl_benches!(
  bench_pollard_rho_128,
  "pollard_rho_128",
  Values::I64(263_5722_1265_1198_9555)
);
impl_benches!(
  bench_snappy_compress,
  "snappy_compress",
  Values::I32(39_3476)
);
