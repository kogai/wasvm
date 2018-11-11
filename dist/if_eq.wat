(module
  (type (;0;) (func (param i32) (result i32)))
  (func (;0;) (type 0) (param i32) (result i32)
    i32.const 5
    i32.const 10
    get_local 0
    i32.const 10
    i32.eq
    select
    get_local 0
    i32.add)
  (export "_subject" (func 0)))
