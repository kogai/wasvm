use inst::Trap;
use std::ops::{BitAnd, BitOr, BitXor};

#[derive(Debug, PartialEq, Clone)]
pub enum Values {
  I32(i32),
  I64(i64),
  // F32,
  // F64,
}

macro_rules! unary_instruction {
  ($fn_name: ident,$op: ident) => {
    pub fn $fn_name(&self) -> Self {
      match self {
        Values::I32(l) => Values::I32(l.$op()),
        Values::I64(l) => Values::I64(l.$op()),
      }
    }
  };
}

macro_rules! numeric_instrunction {
  ($fn_name: ident,$op: ident) => {
    pub fn $fn_name(&self, other: &Self) -> Self {
      match (self, other) {
        (Values::I32(l), Values::I32(r)) => Values::I32(l.$op(*r)),
        (Values::I64(l), Values::I64(r)) => Values::I64(l.$op(*r)),
        _ => unimplemented!(),
      }
    }
  };
}

trait Arithmetic {
  fn equal_zero(&self) -> Self;
  fn count_leading_zero(&self) -> Self;
  fn count_trailing_zero(&self) -> Self;
  fn pop_count(&self) -> Self;

  fn less_than(&self, other: Self) -> Self;
  fn less_than_equal(&self, other: Self) -> Self;
  fn less_than_unsign(&self, other: Self) -> Self;
  fn less_than_equal_unsign(&self, other: Self) -> Self;

  fn greater_than(&self, other: Self) -> Self;
  fn greater_than_equal(&self, other: Self) -> Self;
  fn greater_than_unsign(&self, other: Self) -> Self;
  fn greater_than_equal_unsign(&self, other: Self) -> Self;

  fn equal(&self, other: Self) -> Self;
  fn not_equal(&self, other: Self) -> Self;

  fn shift_left(&self, other: Self) -> Self;
  fn shift_right_sign(&self, other: Self) -> Self;
  fn shift_right_unsign(&self, other: Self) -> Self;

  fn wasm_rotate_left(&self, other: Self) -> Self;
  fn wasm_rotate_right(&self, other: Self) -> Self;
}

macro_rules! impl_traits {
  ($ty: ty, $unsign: ty) => {
    impl Arithmetic for $ty {
      fn equal_zero(&self) -> Self {
        if self == &0 {
          1
        } else {
          0
        }
      }
      fn count_leading_zero(&self) -> Self {
        self.leading_zeros() as $ty
      }
      fn count_trailing_zero(&self) -> Self {
        self.trailing_zeros() as $ty
      }
      fn pop_count(&self) -> Self {
        self.count_ones() as $ty
      }

      fn less_than(&self, other: Self) -> Self {
        if self.lt(&other) {
          1
        } else {
          0
        }
      }
      fn less_than_equal(&self, other: Self) -> Self {
        if self.le(&other) {
          1
        } else {
          0
        }
      }
      fn less_than_unsign(&self, other: Self) -> Self {
        let l1 = *self as $unsign;
        let r1 = other as $unsign;
        if l1.lt(&r1) {
          1
        } else {
          0
        }
      }
      fn less_than_equal_unsign(&self, other: Self) -> Self {
        let l1 = *self as $unsign;
        let r1 = other as $unsign;
        if l1.le(&r1) {
          1
        } else {
          0
        }
      }
      fn greater_than(&self, other: Self) -> Self {
        if self.gt(&other) {
          1
        } else {
          0
        }
      }
      fn greater_than_equal(&self, other: Self) -> Self {
        if self.ge(&other) {
          1
        } else {
          0
        }
      }
      fn greater_than_unsign(&self, other: Self) -> Self {
        let l1 = *self as $unsign;
        let r1 = other as $unsign;
        let result = l1.gt(&r1);
        if result {
          1
        } else {
          0
        }
      }

      fn greater_than_equal_unsign(&self, other: Self) -> Self {
        let l1 = *self as $unsign;
        let r1 = other as $unsign;
        let result = l1.ge(&r1);
        if result {
          1
        } else {
          0
        }
      }
      fn equal(&self, other: Self) -> Self {
        if self.eq(&other) {
          1
        } else {
          0
        }
      }
      fn not_equal(&self, other: Self) -> Self {
        if self.ne(&other) {
          1
        } else {
          0
        }
      }
      fn shift_left(&self, other: Self) -> Self {
        self.wrapping_shl(other as u32)
      }
      fn shift_right_sign(&self, other: Self) -> Self {
        let shifted = self.wrapping_shr(other as u32);
        let casted = (shifted as $unsign) as $ty;
        casted
      }
      fn shift_right_unsign(&self, other: Self) -> Self {
        let i1 = *self as $unsign;
        let shifted = i1.wrapping_shr(other as u32) as $ty;
        shifted
      }

      fn wasm_rotate_left(&self, other: Self) -> Self {
        self.rotate_left(other as u32)
      }

      fn wasm_rotate_right(&self, other: Self) -> Self {
        self.rotate_right(other as u32)
      }
    }
  };
}

impl_traits!(i32, u32);
impl_traits!(i64, u64);

impl Values {
  numeric_instrunction!(and, bitand);
  numeric_instrunction!(or, bitor);
  numeric_instrunction!(xor, bitxor);
  numeric_instrunction!(add, wrapping_add);
  numeric_instrunction!(sub, wrapping_sub);
  numeric_instrunction!(mul, wrapping_mul);

  numeric_instrunction!(less_than, less_than);
  numeric_instrunction!(less_than_equal, less_than_equal);
  numeric_instrunction!(less_than_unsign, less_than_unsign);
  numeric_instrunction!(less_than_equal_unsign, less_than_equal_unsign);

  numeric_instrunction!(greater_than, greater_than);
  numeric_instrunction!(greater_than_equal, greater_than_equal);
  numeric_instrunction!(greater_than_unsign, greater_than_unsign);
  numeric_instrunction!(greater_than_equal_unsign, greater_than_equal_unsign);
  numeric_instrunction!(equal, equal);
  numeric_instrunction!(not_equal, not_equal);

  numeric_instrunction!(shift_left, shift_left);
  numeric_instrunction!(shift_right_sign, shift_right_sign);
  numeric_instrunction!(shift_right_unsign, shift_right_unsign);
  numeric_instrunction!(wasm_rotate_left, wasm_rotate_left);
  numeric_instrunction!(wasm_rotate_right, wasm_rotate_right);

  unary_instruction!(equal_zero, equal_zero);
  unary_instruction!(count_leading_zero, count_leading_zero);
  unary_instruction!(count_trailing_zero, count_trailing_zero);
  unary_instruction!(pop_count, pop_count);

  pub fn rem_s(&self, other: &Self) -> Result<Self, Trap> {
    match (self, other) {
      (Values::I32(l), Values::I32(r)) => {
        if *r == 0 {
          return Err(Trap::DivisionByZero);
        }
        let (divined, overflowed) = l.overflowing_rem(*r);
        if overflowed {
          Err(Trap::DivisionOverflow)
        } else {
          Ok(Values::I32(divined))
        }
      }
      // (Values::I64(l), Values::I64(r)) => Values::I64(l.overflowing_div(*r).0),
      _ => unimplemented!(),
    }
  }

  pub fn rem_u(&self, other: &Self) -> Result<Self, Trap> {
    match (self, other) {
      (Values::I32(l), Values::I32(r)) => {
        if *r == 0 {
          return Err(Trap::DivisionByZero);
        }
        let (divined, overflowed) = (*l as u32).overflowing_rem(*r as u32);
        if overflowed {
          Err(Trap::DivisionOverflow)
        } else {
          Ok(Values::I32(divined as i32))
        }
      }
      // (Values::I64(l), Values::I64(r)) => Values::I64(l.overflowing_div(*r).0),
      _ => unimplemented!(),
    }
  }

  pub fn div_u(&self, other: &Self) -> Result<Self, Trap> {
    match (self, other) {
      (Values::I32(l), Values::I32(r)) => {
        if *r == 0 {
          return Err(Trap::DivisionByZero);
        }
        let (divined, overflowed) = (*l as u32).overflowing_div(*r as u32);
        if overflowed {
          Err(Trap::DivisionOverflow)
        } else {
          Ok(Values::I32(divined as i32))
        }
      }
      // (Values::I64(l), Values::I64(r)) => Values::I64(l.overflowing_div(*r).0),
      _ => unimplemented!(),
    }
  }

  pub fn div_s(&self, other: &Self) -> Result<Self, Trap> {
    match (self, other) {
      (Values::I32(l), Values::I32(r)) => {
        if *r == 0 {
          return Err(Trap::DivisionByZero);
        }
        let (divined, overflowed) = l.overflowing_div(*r);
        if overflowed {
          Err(Trap::DivisionOverflow)
        } else {
          Ok(Values::I32(divined))
        }
      }
      // (Values::I64(l), Values::I64(r)) => Values::I64(l.overflowing_div(*r).0),
      _ => unimplemented!(),
    }
  }

  pub fn is_truthy(&self) -> bool {
    match &self {
      Values::I32(n) => *n > 0,
      _ => unimplemented!(),
    }
  }

  pub fn extend_to_i64(&self) -> Self {
    match self {
      Values::I32(l) => Values::I64(i64::from(*l)),
      _ => unimplemented!(),
    }
  }
}
