#![allow(clippy::needless_range_loop)]
#![feature(test)]
extern crate flame;
extern crate test;
extern crate wasvm;

use std::fs;
use std::io::Read;
use wasvm::{
  decode_module, init_store, instantiate_module, ExternalModule, ExternalModules, FunctionInstance,
  FunctionType, ValueTypes, Values,
};

#[derive(Clone)]
struct Sample(usize, usize);

fn my_hal_function(_arguments: &[Values]) -> Vec<Values> {
  [Values::I32(3 * 5)].to_vec()
}

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
fn memory_alloc(b: &mut test::Bencher) {
  let mut file = fs::File::open("./discovery/src/discovery_wasm_bg.wasm").unwrap();
  let mut bytes = vec![];
  file.read_to_end(&mut bytes).unwrap();
  b.iter(|| {
    let store = init_store();
    let module = decode_module(&bytes);
    let mut external_modules = ExternalModules::default();
    let external_module = ExternalModule::new(
      [FunctionInstance::new_host_fn(
        Some("__wbg_myhalfunction_59a89d8df8955cf7".to_owned()),
        FunctionType::new(
          [ValueTypes::I32, ValueTypes::I32].to_vec(),
          [ValueTypes::I32].to_vec(),
        ),
        &my_hal_function,
      )]
      .to_vec(),
      [].to_vec(),
      [].to_vec(),
      [].to_vec(),
      [].to_vec(),
    );
    external_modules
      .register_module(Some("./discovery_wasm".to_owned()), external_module)
      .unwrap();
    let mut vm = instantiate_module(store, module, external_modules, 65536).unwrap();
    let result = vm.run(
      "use_hal_function",
      [Values::I32(3), Values::I32(5)].to_vec(),
    );
    assert_eq!(result, Ok(Values::I32(25)));
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
