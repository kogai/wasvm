(module
  (type (;0;) (func (result i32)))
  (type (;1;) (func (param i32) (result i32)))
  (import "env" "STACKTOP" (global (;0;) i32))
  (func (;0;) (type 0) (result i32)
    i32.const 42)
  (func (;1;) (type 1) (param i32) (result i32)
    (local i32 i32)
    get_global 1
    set_local 2
    get_global 1
    get_local 0
    i32.add
    set_global 1
    get_global 1
    i32.const 15
    i32.add
    i32.const -16
    i32.and
    set_global 1
    get_local 2)
  (global (;1;) (mut i32) (get_global 0))
  (export "_main" (func 0))
  (export "stackAlloc" (func 1)))
