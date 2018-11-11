(module
  (type (;0;) (func (param i32) (result i32)))
  (func (;0;) (type 0) (param i32) (result i32)
    (local i32)
    get_local 0
    i32.const 2
    i32.lt_u
    if (result i32)  ;; label = @1
      get_local 0
    else
      get_local 0
      i32.const -2
      i32.add
      call 0
      set_local 1
      get_local 0
      i32.const -1
      i32.add
      call 0
      get_local 1
      i32.add
    end)
  (func (;1;) (type 0) (param i32) (result i32)
    (local i32)
    get_local 0
    i32.const 1
    i32.add
    call 0
    set_local 1
    get_local 0
    i32.const 2
    i32.add
    call 0
    get_local 1
    i32.add)
  (export "_subject" (func 1)))
