[![Build Status](https://travis-ci.org/kogai/wasvm.svg?branch=master)](https://travis-ci.org/kogai/wasvm)

## Status

- All `assert_return` and several tests of testsuite which WASM core team provided has been passed.

## TODO

- [ ] Show position where error occurred at docoding time
- [ ] Implement type system(In definitions of WASM semantics, validation)
- [ ] Investigate to introduce JIT
- [ ] Consider something which this implementation fit to
- [x] Consider to measure performance
  - https://blog.sqreen.io/webassembly-performance/
  - https://github.com/perlin-network/life/tree/master/bench/cases

## Debug

```sh
$ gdb ./target/debug/wasvm-{.*}
run --test test_name
break src/lib.rs:160
info locals
info breakpoint
delete ${idx}
```
