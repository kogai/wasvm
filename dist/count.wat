(module
  (type (;0;) (func (param i32) (result i32)))
  (func (;0;) (type 0) (param i32) (result i32)
    (local i32)
    get_local 0
    i32.const 0
    i32.le_s
    if  ;; label = @1
      i32.const 0
      return
    end
    get_local 0
    i32.const -1
    i32.add
    tee_local 1
    get_local 0
    i32.const 1
    i32.add
    i32.mul
    get_local 0
    i32.add
    get_local 1
    i64.extend_u/i32
    get_local 0
    i32.const -2
    i32.add
    i64.extend_u/i32
    i64.mul
    i64.const 8589934591
    i64.and
    i64.const 1
    i64.shr_u
    i32.wrap/i64
    i32.add)
  (export "_subject" (func 0)))
