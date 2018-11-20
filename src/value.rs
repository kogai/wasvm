use inst::Trap;
use std::ops::{BitAnd, BitOr, BitXor};

#[derive(Debug, PartialEq, Clone)]
pub enum Values {
  I32(i32),
  I64(i64),
  // F32,
  // F64,
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

macro_rules! conditional_instrunction {
  ($fn_name: ident,$op: ident) => {
    pub fn $fn_name(&self, other: &Self) -> Self {
      match (self, other) {
        (Values::I32(l), Values::I32(r)) => Values::I32(if l.$op(r) { 1 } else { 0 }),
        (Values::I64(l), Values::I64(r)) => Values::I64(if l.$op(r) { 1 } else { 0 }),
        _ => unimplemented!(),
      }
    }
  };
}

impl Values {
  conditional_instrunction!(less_than, lt);
  conditional_instrunction!(less_than_equal, le);
  conditional_instrunction!(greater_than, gt);
  conditional_instrunction!(greater_than_equal, ge);
  conditional_instrunction!(equal, eq);
  conditional_instrunction!(not_equal, ne);

  numeric_instrunction!(and, bitand);
  numeric_instrunction!(or, bitor);
  numeric_instrunction!(xor, bitxor);
  numeric_instrunction!(add, wrapping_add);
  numeric_instrunction!(sub, wrapping_sub);
  numeric_instrunction!(mul, wrapping_mul);

  pub fn equal_zero(&self) -> Self {
    match self {
      Values::I32(n) => Values::I32(if *n == 0 { 1 } else { 0 }),
      _ => unimplemented!(),
    }
  }

  pub fn less_than_unsign(&self, other: &Self) -> Self {
    match (self, other) {
      (Values::I32(l), Values::I32(r)) => {
        let l1 = *l as u32;
        let r1 = *r as u32;
        let result = l1.lt(&r1);
        Values::I32(if result { 1 } else { 0 })
      }
      _ => unimplemented!(),
    }
  }

  pub fn less_than_equal_unsign(&self, other: &Self) -> Self {
    match (self, other) {
      (Values::I32(l), Values::I32(r)) => {
        let l1 = *l as u32;
        let r1 = *r as u32;
        let result = l1.le(&r1);
        Values::I32(if result { 1 } else { 0 })
      }
      _ => unimplemented!(),
    }
  }

  pub fn greater_than_unsign(&self, other: &Self) -> Self {
    match (self, other) {
      (Values::I32(l), Values::I32(r)) => {
        let l1 = *l as u32;
        let r1 = *r as u32;
        let result = l1.gt(&r1);
        Values::I32(if result { 1 } else { 0 })
      }
      _ => unimplemented!(),
    }
  }

  pub fn greater_than_equal_unsign(&self, other: &Self) -> Self {
    match (self, other) {
      (Values::I32(l), Values::I32(r)) => {
        let l1 = *l as u32;
        let r1 = *r as u32;
        let result = l1.ge(&r1);
        Values::I32(if result { 1 } else { 0 })
      }
      _ => unimplemented!(),
    }
  }

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

  pub fn shift_left(&self, other: &Self) -> Self {
    match (self, other) {
      (Values::I32(i1), Values::I32(i2)) => {
        let shifted = i1.wrapping_shl(*i2 as u32);
        Values::I32(shifted)
      }
      _ => unimplemented!(),
    }
  }

  pub fn shift_right_sign(&self, other: &Self) -> Self {
    match (self, other) {
      (Values::I32(i1), Values::I32(i2)) => {
        let shifted = i1.wrapping_shr(*i2 as u32);
        Values::I32((shifted as u32) as i32)
      }
      _ => unimplemented!(),
    }
  }

  pub fn shift_right_unsign(&self, other: &Self) -> Self {
    match (self, other) {
      (Values::I32(i1), Values::I32(i2)) => {
        let i1 = *i1 as u32;
        let i2 = *i2 as u32;
        let shifted = i1.wrapping_shr(i2) as i32;
        Values::I32(shifted)
      }
      (Values::I64(i1), Values::I64(i2)) => {
        let i1 = *i1 as u64;
        let i2 = *i2 as u64;
        let shifted = i1.wrapping_shr((i2 % 64) as u32) as i64;
        Values::I64(shifted)
      }
      _ => unimplemented!(),
    }
  }

  pub fn rotate_left(&self, other: &Self) -> Self {
    match (self, other) {
      (Values::I32(i1), Values::I32(i2)) => Values::I32(i1.rotate_left(*i2 as u32)),
      _ => unimplemented!(),
    }
  }

  pub fn rotate_right(&self, other: &Self) -> Self {
    match (self, other) {
      (Values::I32(i1), Values::I32(i2)) => Values::I32(i1.rotate_right(*i2 as u32)),
      _ => unimplemented!(),
    }
  }

  pub fn is_truthy(&self) -> bool {
    match &self {
      Values::I32(n) => *n > 0,
      _ => unimplemented!(),
    }
  }

  pub fn count_leading_zero(&self) -> Self {
    match self {
      Values::I32(l) => Values::I32(l.leading_zeros() as i32),
      _ => unimplemented!(),
    }
  }

  pub fn count_trailing_zero(&self) -> Self {
    match self {
      Values::I32(l) => Values::I32(l.trailing_zeros() as i32),
      _ => unimplemented!(),
    }
  }

  pub fn pop_count(&self) -> Self {
    match self {
      Values::I32(l) => Values::I32(l.count_ones() as i32),
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
