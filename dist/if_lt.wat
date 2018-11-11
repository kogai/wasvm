(module
  (type (;0;) (func (param i32) (result i32)))
  (func (;0;) (type 0) (param i32) (result i32)
    get_local 0
    i32.const 10
    i32.add
    i32.const 15
    get_local 0
    i32.const 15
    i32.add
    get_local 0
    i32.const 10
    i32.eq
    select
    get_local 0
    i32.const 10
    i32.lt_s
    select)
  (export "_subject" (func 0)))
