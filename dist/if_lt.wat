(module
  (type (;0;) (func (param i32) (result i32)))
  (func (;0;) (type 0) (param i32) (result i32)
    (local i32)
    get_local 0
    i32.const 10
    i32.lt_s
    if (result i32)  ;; label = @1
      get_local 0
      i32.const 10
      i32.add
    else
      get_local 0
      i32.const 15
      i32.add
      set_local 1
      get_local 0
      i32.const 10
      i32.eq
      if (result i32)  ;; label = @2
        i32.const 15
      else
        get_local 1
      end
    end)
  (export "_subject" (func 0)))
