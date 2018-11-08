(module
  (type (;0;) (func (param i32 i32) (result i32)))
  (func (;0;) (type 0) (param i32 i32) (result i32)
    get_local 1
    get_local 0
    i32.add)
  (func (;1;) (type 0) (param i32 i32) (result i32)
    get_local 0
    get_local 1
    i32.sub)
  (export "_f" (func 1))
  (export "_g" (func 0)))
