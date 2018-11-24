[![Build Status](https://travis-ci.org/kogai/vm.svg?branch=master)](https://travis-ci.org/kogai/vm)

## TODO

* [ ] Simplify read process of the WASM file
* [ ] Simplify evaluate_instructions
* [ ] Test assert_trap and assert_malformed
* [ ] Return Result from evaluate_inst
* [ ] Show position where error occurred at docoding time

## Debug

```sh
$ gdb ./target/debug/wasvm-{.*}
run --test test_name
break src/lib.rs:160
info locals
info breakpoint
delete ${idx}

$ perf record cargo run --release fib 30
$ perf report
```

## Performance index

https://blog.sqreen.io/webassembly-performance/
