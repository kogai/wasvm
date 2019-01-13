use alloc::prelude::*;
use alloc::string::String;
use core::mem::transmute;
use core::ops::Rem;
use core::ops::{BitAnd, BitOr, BitXor, Neg};
use core::{f32, f64, fmt};
use trap::{Result, Trap};
use value_type::ValueTypes;

#[derive(PartialEq, Clone)]
pub enum Values {
  I32(i32),
  I64(i64),
  F32(f32),
  F64(f64),
}

macro_rules! unary_inst {
  ($fn_name: ident,$op: ident) => {
    pub fn $fn_name(&self) -> Self {
      match self {
        Values::I32(l) => Values::I32(l.$op()),
        Values::I64(l) => Values::I64(l.$op()),
        Values::F32(l) => Values::F32(l.$op()),
        Values::F64(l) => Values::F64(l.$op()),
      }
    }
  };
}

macro_rules! unary_logical_inst {
  ($fn_name: ident,$op: ident) => {
    pub fn $fn_name(&self) -> Self {
      match self {
        Values::I32(l) => Values::I32(l.$op()),
        Values::I64(l) => Values::I32(l.$op() as i32),
        Values::F32(l) => Values::I32(l.$op() as i32),
        Values::F64(l) => Values::I32(l.$op() as i32),
      }
    }
  };
}

macro_rules! binary_inst {
  ($fn_name: ident,$op: ident) => {
    pub fn $fn_name(&self, other: &Self) -> Self {
      match (self, other) {
        (Values::I32(l), Values::I32(r)) => Values::I32(l.$op(*r)),
        (Values::I64(l), Values::I64(r)) => Values::I64(l.$op(*r)),
        (Values::F32(l), Values::F32(r)) => Values::F32(l.$op(r.to_owned())),
        (Values::F64(l), Values::F64(r)) => Values::F64(l.$op(r.to_owned())),
        _ => unimplemented!(),
      }
    }
  };
}

macro_rules! binary_logical_inst {
  ($fn_name: ident,$op: ident) => {
    pub fn $fn_name(&self, other: &Self) -> Self {
      match (self, other) {
        (Values::I32(l), Values::I32(r)) => Values::I32(l.$op(*r)),
        (Values::I64(l), Values::I64(r)) => Values::I32(l.$op(*r) as i32),
        (Values::F32(l), Values::F32(r)) => Values::I32(l.$op(*r) as i32),
        (Values::F64(l), Values::F64(r)) => Values::I32(l.$op(*r) as i32),
        _ => unimplemented!(),
      }
    }
  };
}

macro_rules! binary_try_inst {
  ($fn_name: ident,$op: ident) => {
    pub fn $fn_name(&self, other: &Self) -> Result<Self> {
      match (self, other) {
        (Values::I32(l), Values::I32(r)) =>  l.$op(*r).map(Values::I32),
        (Values::I64(l), Values::I64(r)) =>  l.$op(*r).map(Values::I64),
        _ => unimplemented!(),
      }
    }
  };
}

trait Arithmetic {}

macro_rules! impl_traits {
  ($ty: ty) => {
    impl Arithmetic for $ty {}
  };
}

trait ArithmeticInteger {
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

  fn rem_s(&self, other: Self) -> Result<Self>
  where
    Self: Sized;
  fn rem_u(&self, other: Self) -> Result<Self>
  where
    Self: Sized;
  fn div_s(&self, other: Self) -> Result<Self>
  where
    Self: Sized;
  fn div_u(&self, other: Self) -> Result<Self>
  where
    Self: Sized;
  fn copy_sign(&self, other: Self) -> Self;
}

macro_rules! impl_integer_traits {
  ($ty: ty, $unsign: ty) => {
    impl ArithmeticInteger for $ty {
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
        (shifted as $unsign) as $ty
      }
      fn shift_right_unsign(&self, other: Self) -> Self {
        let i1 = *self as $unsign;
        i1.wrapping_shr(other as u32) as $ty
      }

      fn wasm_rotate_left(&self, other: Self) -> Self {
        self.rotate_left(other as u32)
      }

      fn wasm_rotate_right(&self, other: Self) -> Self {
        self.rotate_right(other as u32)
      }

      fn rem_s(&self, other: Self) -> Result<Self> {
        if other == 0 {
          return Err(Trap::DivisionByZero);
        }
        let (divined, _) = self.overflowing_rem(other);
        Ok(divined)
      }

      fn rem_u(&self, other: Self) -> Result<Self> {
        if other == 0 {
          return Err(Trap::DivisionByZero);
        }
        let (divined, overflowed) = (*self as $unsign).overflowing_rem(other as $unsign);
        if overflowed {
          Err(Trap::DivisionOverflow)
        } else {
          Ok(divined as $ty)
        }
      }
      fn div_u(&self, other: Self) -> Result<Self> {
        if other == 0 {
          return Err(Trap::DivisionByZero);
        }
        let (divined, overflowed) = (*self as $unsign).overflowing_div(other as $unsign);
        if overflowed {
          Err(Trap::DivisionOverflow)
        } else {
          Ok(divined as $ty)
        }
      }
      fn div_s(&self, other: Self) -> Result<Self> {
        if other == 0 {
          return Err(Trap::DivisionByZero);
        }
        let (divined, overflowed) = self.overflowing_div(other);
        if overflowed {
          Err(Trap::DivisionOverflow)
        } else {
          Ok(divined)
        }
      }
      fn copy_sign(&self, other: Self) -> Self {
        if self.signum() == other.signum() {
          *self
        } else {
          self.neg()
        }
      }
    }
  };
}

trait ArithmeticFloat {
  fn bitand(&self, _: Self) -> Self
  where
    Self: Sized,
  {
    unreachable!();
  }
  fn bitor(&self, _: Self) -> Self
  where
    Self: Sized,
  {
    unreachable!();
  }
  fn bitxor(&self, _: Self) -> Self
  where
    Self: Sized,
  {
    unreachable!();
  }
  fn less_than_unsign(&self, Self) -> Self
  where
    Self: Sized,
  {
    unreachable!();
  }

  fn less_than_equal_unsign(&self, Self) -> Self
  where
    Self: Sized,
  {
    unreachable!();
  }

  fn greater_than_unsign(&self, Self) -> Self
  where
    Self: Sized,
  {
    unreachable!();
  }

  fn greater_than_equal_unsign(&self, Self) -> Self
  where
    Self: Sized,
  {
    unreachable!();
  }

  fn equal_zero(&self) -> i32;
  fn count_leading_zero(&self) -> Self;
  fn count_trailing_zero(&self) -> Self;
  fn pop_count(&self) -> Self;

  fn wrapping_add(&self, _: Self) -> Self;
  fn wrapping_sub(&self, _: Self) -> Self;
  fn wrapping_mul(&self, _: Self) -> Self;
  fn less_than(&self, Self) -> Self;
  fn less_than_equal(&self, Self) -> Self;
  fn greater_than(&self, Self) -> Self;
  fn greater_than_equal(&self, Self) -> Self;
  fn equal(&self, Self) -> Self;
  fn not_equal(&self, Self) -> Self;
  fn shift_left(&self, Self) -> Self;
  fn shift_right_sign(&self, Self) -> Self;
  fn shift_right_unsign(&self, Self) -> Self;
  fn wasm_rotate_left(&self, Self) -> Self;
  fn wasm_rotate_right(&self, Self) -> Self;
  fn copy_sign(&self, Self) -> Self;
}

macro_rules! impl_float_traits {
  ($ty: ty) => {
    impl ArithmeticFloat for $ty {
      fn equal_zero(&self) -> i32 {
        if *self == 0.0 {
          1
        } else {
          0
        }
      }
      fn count_leading_zero(&self) -> Self {
        unreachable!();
      }
      fn count_trailing_zero(&self) -> Self {
        unreachable!();
      }
      fn pop_count(&self) -> Self {
        unreachable!();
      }
      fn wrapping_add(&self, x: Self) -> Self {
        self + x
      }
      fn wrapping_sub(&self, x: Self) -> Self {
        self - x
      }
      fn wrapping_mul(&self, x: Self) -> Self {
        self * x
      }
      fn less_than(&self, x: Self) -> Self {
        if self < &x {
          1.0
        } else {
          0.0
        }
      }
      fn less_than_equal(&self, x: Self) -> Self {
        if self <= &x {
          1.0
        } else {
          0.0
        }
      }
      fn greater_than(&self, x: Self) -> Self {
        if self > &x {
          1.0
        } else {
          0.0
        }
      }
      fn greater_than_equal(&self, x: Self) -> Self {
        if self >= &x {
          1.0
        } else {
          0.0
        }
      }
      fn equal(&self, x: Self) -> Self {
        if self.eq(&x) {
          1.0
        } else {
          0.0
        }
      }
      fn not_equal(&self, x: Self) -> Self {
        if self.ne(&x) {
          1.0
        } else {
          0.0
        }
      }
      fn shift_left(&self, _x: Self) -> Self {
        unreachable!();
      }
      fn shift_right_sign(&self, _x: Self) -> Self {
        unreachable!();
      }
      fn shift_right_unsign(&self, _x: Self) -> Self {
        unreachable!();
      }
      fn wasm_rotate_left(&self, _x: Self) -> Self {
        unreachable!();
      }
      fn wasm_rotate_right(&self, _x: Self) -> Self {
        unreachable!();
      }
      fn copy_sign(&self, other: Self) -> Self {
        if (self.is_sign_positive() == other.is_sign_positive())
          || (self.is_sign_negative() == other.is_sign_negative())
        {
          *self
        } else {
          -*self
        }
      }
    }
  };
}

trait TruncFloat<T> {
  fn try_trunc_to(&self) -> Result<T>;
}

macro_rules! impl_try_trunc {
  ($from: ty, $to: ty) => {
    impl TruncFloat<$to> for $from {
      fn try_trunc_to(&self) -> Result<$to> {
        if self.is_nan() {
          return Err(Trap::InvalidConversionToInt);
        }
        if self.is_infinite() {
          return Err(Trap::IntegerOverflow);
        }
        let result = *self as $to;
        if (result as $from).ne(&self.trunc()) {
          return Err(Trap::IntegerOverflow);
        }
        Ok(result as $to)
      }
    }
  };
}

macro_rules! trunc_inst {
  ($name: ident, $kind_from: path, $kind_to: path, $internal: ty, $to: ty) => {
      pub fn $name(&self) -> Result<Self> {
        match self {
          $kind_from(n) => {
            let result: $internal = n.try_trunc_to()?;
            Ok($kind_to(result as $to))
          }
          x => unreachable!("Got {:?}", x),
        }
      }
  };
}

impl_traits!(i32);
impl_traits!(i64);
impl_traits!(f32);
impl_traits!(f64);

impl_integer_traits!(i32, u32);
impl_integer_traits!(i64, u64);
impl_float_traits!(f32);
impl_float_traits!(f64);

impl_try_trunc!(f32, i32);
impl_try_trunc!(f32, u32);
impl_try_trunc!(f32, i64);
impl_try_trunc!(f32, u64);
impl_try_trunc!(f64, i32);
impl_try_trunc!(f64, u32);
impl_try_trunc!(f64, i64);
impl_try_trunc!(f64, u64);

impl Values {
  binary_inst!(and, bitand);
  binary_inst!(or, bitor);
  binary_inst!(xor, bitxor);
  binary_inst!(add, wrapping_add);
  binary_inst!(sub, wrapping_sub);
  binary_inst!(mul, wrapping_mul);

  binary_logical_inst!(less_than, less_than);
  binary_logical_inst!(less_than_equal, less_than_equal);
  binary_logical_inst!(less_than_unsign, less_than_unsign);
  binary_logical_inst!(less_than_equal_unsign, less_than_equal_unsign);

  binary_logical_inst!(greater_than, greater_than);
  binary_logical_inst!(greater_than_equal, greater_than_equal);
  binary_logical_inst!(greater_than_unsign, greater_than_unsign);
  binary_logical_inst!(greater_than_equal_unsign, greater_than_equal_unsign);
  binary_logical_inst!(equal, equal);
  binary_logical_inst!(not_equal, not_equal);

  binary_inst!(shift_left, shift_left);
  binary_inst!(shift_right_sign, shift_right_sign);
  binary_inst!(shift_right_unsign, shift_right_unsign);
  binary_inst!(wasm_rotate_left, wasm_rotate_left);
  binary_inst!(wasm_rotate_right, wasm_rotate_right);
  binary_inst!(copy_sign, copy_sign);

  binary_try_inst!(rem_s, rem_s);
  binary_try_inst!(rem_u, rem_u);
  binary_try_inst!(div_s, div_s);
  binary_try_inst!(div_u, div_u);

  unary_logical_inst!(equal_zero, equal_zero);
  unary_inst!(count_leading_zero, count_leading_zero);
  unary_inst!(count_trailing_zero, count_trailing_zero);
  unary_inst!(pop_count, pop_count);
  unary_inst!(abs, abs);
  unary_inst!(neg, neg);

  pub fn promote_f32_to_f64(&self) -> Self {
    match &self {
      Values::F32(n) => {
        if n.is_nan() {
          Values::F64(f64::NAN)
        } else {
          Values::F64(f64::from(*n))
        }
      }
      _ => unreachable!(),
    }
  }

  pub fn demote_f64_to_f32(&self) -> Self {
    match &self {
      Values::F64(n) => {
        if n.is_nan() {
          Values::F32(f32::NAN)
        } else {
          Values::F32(*n as f32)
        }
      }
      _ => unreachable!(),
    }
  }

  pub fn convert_sign_i32_to_f32(&self) -> Self {
    match self {
      Values::I32(n) => Values::F32(*n as f32),
      _ => unreachable!(),
    }
  }
  pub fn convert_unsign_i32_to_f32(&self) -> Self {
    match self {
      Values::I32(n) => Values::F32((*n as u32) as f32),
      _ => unreachable!(),
    }
  }
  pub fn convert_sign_i64_to_f64(&self) -> Self {
    match self {
      Values::I64(n) => Values::F64(*n as f64),
      _ => unreachable!(),
    }
  }
  pub fn convert_unsign_i64_to_f64(&self) -> Self {
    match self {
      Values::I64(n) => Values::F64((*n as u64) as f64),
      _ => unreachable!(),
    }
  }
  pub fn convert_sign_i32_to_f64(&self) -> Self {
    match self {
      Values::I32(n) => Values::F64(f64::from(*n)),
      _ => unreachable!(),
    }
  }
  pub fn convert_unsign_i32_to_f64(&self) -> Self {
    match self {
      Values::I32(n) => Values::F64(f64::from(*n as u32)),
      _ => unreachable!(),
    }
  }
  pub fn convert_sign_i64_to_f32(&self) -> Self {
    match self {
      Values::I64(n) => Values::F32(*n as f32),
      _ => unreachable!(),
    }
  }
  pub fn convert_unsign_i64_to_f32(&self) -> Self {
    match self {
      Values::I64(n) => Values::F32((*n as u64) as f32),
      _ => unreachable!(),
    }
  }

  trunc_inst!(trunc_f32_to_sign_i32, Values::F32, Values::I32, i32, i32);
  trunc_inst!(trunc_f32_to_unsign_i32, Values::F32, Values::I32, u32, i32);
  trunc_inst!(trunc_f64_to_sign_i32, Values::F64, Values::I32, i32, i32);
  trunc_inst!(trunc_f64_to_unsign_i32, Values::F64, Values::I32, u32, i32);
  trunc_inst!(trunc_f32_to_sign_i64, Values::F32, Values::I64, i64, i64);
  trunc_inst!(trunc_f32_to_unsign_i64, Values::F32, Values::I64, u64, i64);
  trunc_inst!(trunc_f64_to_sign_i64, Values::F64, Values::I64, i64, i64);
  trunc_inst!(trunc_f64_to_unsign_i64, Values::F64, Values::I64, u64, i64);

  pub fn reinterpret(&self) -> Self {
    match self {
      Values::I32(n) => Values::F32(f32::from_bits(*n as u32)),
      Values::I64(n) => Values::F64(f64::from_bits(*n as u64)),
      Values::F32(n) => Values::I32(unsafe { transmute(*n) }),
      Values::F64(n) => Values::I64(unsafe { transmute(*n) }),
    }
  }

  pub fn is_truthy(&self) -> bool {
    match &self {
      Values::I32(n) => *n != 0,
      _ => unimplemented!(),
    }
  }

  pub fn extend_u32_to_i64(&self) -> Self {
    match self {
      Values::I32(l) => Values::I64(i64::from(*l as u32)),
      _ => unimplemented!(),
    }
  }

  pub fn extend_i32_to_i64(&self) -> Self {
    match self {
      Values::I32(l) => Values::I64(i64::from(*l)),
      _ => unimplemented!(),
    }
  }
  pub fn div_f(&self, other: &Self) -> Self {
    match (self, other) {
      (Values::F32(l), Values::F32(r)) => Values::F32(l / *r),
      (Values::F64(l), Values::F64(r)) => Values::F64(l / *r),
      _ => unimplemented!(),
    }
  }
  pub fn min(&self, other: &Self) -> Self {
    match (self, other) {
      (Values::F32(l), Values::F32(r)) => {
        if l.is_nan() || r.is_nan() {
          Values::F32(f32::NAN)
        } else {
          Values::F32(l.min(*r))
        }
      }
      (Values::F64(l), Values::F64(r)) => {
        if l.is_nan() || r.is_nan() {
          Values::F64(f64::NAN)
        } else {
          Values::F64(l.min(*r))
        }
      }
      _ => unimplemented!(),
    }
  }
  pub fn max(&self, other: &Self) -> Self {
    match (self, other) {
      (Values::F32(l), Values::F32(r)) => {
        if l.is_nan() || r.is_nan() {
          Values::F32(f32::NAN)
        } else {
          Values::F32(l.max(*r))
        }
      }
      (Values::F64(l), Values::F64(r)) => {
        if l.is_nan() || r.is_nan() {
          Values::F64(f64::NAN)
        } else {
          Values::F64(l.max(*r))
        }
      }
      _ => unimplemented!(),
    }
  }
  pub fn sqrt(&self) -> Self {
    match self {
      Values::F32(l) => Values::F32(l.sqrt()),
      Values::F64(l) => Values::F64(l.sqrt()),
      _ => unimplemented!(),
    }
  }
  pub fn ceil(&self) -> Self {
    match self {
      Values::F32(l) => Values::F32(l.ceil()),
      Values::F64(l) => Values::F64(l.ceil()),
      _ => unimplemented!(),
    }
  }
  pub fn floor(&self) -> Self {
    match self {
      Values::F32(l) => Values::F32(l.floor()),
      Values::F64(l) => Values::F64(l.floor()),
      _ => unimplemented!(),
    }
  }
  pub fn trunc(&self) -> Self {
    match self {
      Values::F32(l) => Values::F32(l.trunc()),
      Values::F64(l) => Values::F64(l.trunc()),
      _ => unimplemented!(),
    }
  }
  pub fn nearest(&self) -> Self {
    match self {
      Values::F32(l) => {
        if (*l > 0.0 && *l <= 0.5) || (*l < 0.0 && *l >= -0.5) {
          Values::F32(0.0)
        } else {
          let round = l.round();
          let result = if round.rem(2.0).eq(&1.0) {
            l.floor()
          } else if round.rem(2.0).eq(&-1.0) {
            l.ceil()
          } else {
            round
          };
          Values::F32(result)
        }
      }
      Values::F64(l) => {
        if (*l > 0.0 && *l <= 0.5) || (*l < 0.0 && *l >= -0.5) {
          Values::F64(0.0)
        } else {
          let round = l.round();
          let result = if round.rem(2.0).eq(&1.0) {
            l.floor()
          } else if round.rem(2.0).eq(&-1.0) {
            l.ceil()
          } else {
            round
          };
          Values::F64(result)
        }
      }
      _ => unimplemented!(),
    }
  }
}

macro_rules! impl_from_values {
  ($ty: ty) => {
    impl From<$ty> for String {
      fn from(x: $ty) -> Self {
        use Values::*;
        match x {
          I32(n) => format!("i32:{}", n),
          I64(n) => format!("i64:{}", n),
          F32(n) => {
            let prefix = if n.is_nan() { "" } else { "f32:" };
            format!("{}{}", prefix, n)
          }
          F64(n) => {
            let prefix = if n.is_nan() { "" } else { "f64:" };
            format!("{}{}", prefix, n)
          }
        }
        .to_owned()
      }
    }
  };
}

impl_from_values!(Values);
impl_from_values!(&Values);

macro_rules! impl_from_valuetypes {
  ($ty: ty) => {
    impl From<$ty> for Values {
      fn from(x: $ty) -> Self {
        match x {
          ValueTypes::I32 => Values::I32(0),
          ValueTypes::I64 => Values::I64(0),
          ValueTypes::F32 => Values::F32(0.0),
          ValueTypes::F64 => Values::F64(0.0),
          ValueTypes::Empty => unreachable!(),
        }
      }
    }
  };
}

impl_from_valuetypes!(ValueTypes);
impl_from_valuetypes!(&ValueTypes);

impl fmt::Debug for Values {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", String::from(self.to_owned()))
  }
}
