[![Build Status](https://travis-ci.org/kogai/vm.svg?branch=master)](https://travis-ci.org/kogai/vm)

## TODO

* [ ] Test assert_trap and assert_malformed
* [x] Return Result from evaluate_inst
* [ ] Show position where error occurred at docoding time
* [ ] Separate Runtime-error "Trap" and Decoding-time-error
* [x] Build wasm from wast at CI
* [ ] Reasonable, pretty formattable Stack

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
