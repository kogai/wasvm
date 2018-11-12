(module
  (type (;0;) (func (param i32 i32) (result i32)))
  (func (;0;) (type 0) (param i32 i32) (result i32)
    get_local 0
    i32.const 10
    i32.add
    get_local 1
    i32.add)
  (export "_subject" (func 0)))
