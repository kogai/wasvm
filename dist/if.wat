(module
  (type (;0;) (func (param i32) (result i32)))
  (func (;0;) (type 0) (param i32) (result i32)
    get_local 0
    i32.const 100
    get_local 0
    i32.const 10
    i32.lt_s
    select)
  (export "_subject" (func 0)))
