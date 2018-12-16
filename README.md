[![Build Status](https://travis-ci.org/kogai/vm.svg?branch=master)](https://travis-ci.org/kogai/vm)

## TODO

- [x] Test assert_trap and assert_malformed
- [x] Return Result from evaluate_inst
- [ ] Show position where error occurred at docoding time
- [ ] Implement type system
  - [ ] Separate Runtime-error "Trap" and Decoding-time-error
- [x] Build wasm from wast at CI
- [x] Reasonable, pretty formattable Stack
- [ ] Investigate whether a test of block/at-load-operand is correct
  - May need to ask core-team?
- [ ] Consider to measure performance
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
