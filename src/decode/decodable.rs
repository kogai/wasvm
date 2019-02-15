use alloc::string::String;
use alloc::vec::Vec;
use error::runtime::Trap;
use error::{Result, WasmError};
use memory::Limit;

macro_rules! impl_decode_leb128 {
  ($ty: ty, $conv_fn: path, $fn_name: ident) => {
    fn $fn_name(&mut self) -> $crate::error::Result<($ty, u32)> {
      let mut buf: $ty = 0;
      let mut shift = 0;

      // Check whether leftmost bit is 1 or 0, if most significant bit is zero,
      // A result of bitwise AND become zero too.
      //        +------------+------------+
      // N      | 0b11111111 | 0b01111111 |
      // &      +      &     +      &     |
      // B      | 0b10000000 | 0b10000000 |
      //        +------------+------------+
      // Result | 0b10000000 | 0b00000000 |
      //        +------------+------------+
      loop {
        let raw_code = self.next()?;
        let is_msb_zero = raw_code & 0b1000_0000 == 0;
        let num = $conv_fn(raw_code & 0b0111_1111); // Drop leftmost bit
        // buf =      00000000_00000000_10000000_00000000
        // num =      00000000_00000000_00000000_00000001
        // num << 7 = 00000000_00000000_00000000_10000000
        // buf | num  00000000_00000000_10000000_10000000
        let (shifted, is_overflowed) = num.overflowing_shl(shift);
        if is_overflowed {
          return Err($crate::error::WasmError::Trap($crate::error::runtime::Trap::IntegerRepresentationTooLong));
        }
        buf |= shifted;
        shift += 7;
        if is_msb_zero {
          break;
        }
      }
      Ok((buf, shift))
    }
  };
}

macro_rules! impl_decode_signed_integer {
  ($fn_name: ident, $decode_name: ident, $buf_ty: ty) => {
      fn $fn_name(&mut self) -> Result<$buf_ty> {
        let (mut buf, shift) = self.$decode_name()?;
        let (signed_bits, overflowed) = (1 as $buf_ty).overflowing_shl(shift - 1);
        if overflowed {
          return Ok(buf);
        }
        let is_buf_signed = buf & signed_bits != 0;
        if is_buf_signed {
          buf |= !0 << shift;
        };
        Ok(buf)
      }
  };
}

pub trait AbstractDecodable {
  fn bytes(&self) -> &Vec<u8>;
  fn byte_ptr(&self) -> usize;
  fn increment_ptr(&mut self);
}

pub trait U8Iterator: AbstractDecodable {
  fn next(&mut self) -> Option<u8> {
    let el = self.bytes().get(self.byte_ptr()).cloned();
    self.increment_ptr();
    el
  }
}

pub trait Peekable: AbstractDecodable {
  fn peek(&self) -> Option<u8> {
    self.bytes().get(self.byte_ptr()).cloned()
  }
}

pub trait Leb128Decodable: U8Iterator {
  impl_decode_leb128!(u32, u32::from, decode_leb128_u32_internal);
  impl_decode_leb128!(u64, u64::from, decode_leb128_u64_internal);
}

pub trait U32Decodable: Leb128Decodable {
  fn decode_leb128_u32(&mut self) -> Result<u32> {
    let (buf, _) = self.decode_leb128_u32_internal()?;
    Ok(buf)
  }
}

pub trait SignedIntegerDecodable: Leb128Decodable {
  impl_decode_signed_integer!(decode_leb128_i32, decode_leb128_u32_internal, u32);
  impl_decode_signed_integer!(decode_leb128_i64, decode_leb128_u64_internal, u64);
}

pub trait LimitDecodable: U32Decodable {
  fn decode_limit(&mut self) -> Result<Limit> {
    use self::Limit::*;
    match self.next() {
      Some(0x0) => {
        let min = self.decode_leb128_u32()?;
        Ok(NoUpperLimit(min))
      }
      Some(0x1) => {
        let min = self.decode_leb128_u32()?;
        let max = self.decode_leb128_u32()?;
        Ok(HasUpperLimit(min, max))
      }
      x => unreachable!("Expected limit code, got {:?}", x),
    }
  }
}

pub trait NameDecodable: U32Decodable {
  fn decode_name(&mut self) -> Result<String> {
    let size_of_name = self.decode_leb128_u32()?;
    let mut buf = vec![];
    for _ in 0..size_of_name {
      buf.push(self.next()?);
    }
    String::from_utf8(buf).map_err(|_| WasmError::Trap(Trap::InvalidUTF8Encoding))
  }
}

macro_rules! impl_decodable {
  ($name: ident) => {
    pub struct $name {
      bytes: Vec<u8>,
      byte_ptr: usize,
    }

    impl $crate::decode::AbstractDecodable for $name {
      fn bytes(&self) -> &Vec<u8> {
        &self.bytes
      }
      fn byte_ptr(&self) -> usize {
        self.byte_ptr
      }
      fn increment_ptr(&mut self) {
        self.byte_ptr += 1;
      }
    }

    impl $crate::decode::U8Iterator for $name {}

    impl $name {
      pub fn new(bytes: Vec<u8>) -> Self {
        $name {
          bytes: bytes,
          byte_ptr: 0,
        }
      }
    }
  };
}

pub trait Decodable {
  type Item;
  fn decode(&mut self) -> Result<Self::Item>;
}

#[cfg(test)]
mod tests {
  use super::*;

  impl_decodable!(TestDecodable);
  impl Leb128Decodable for TestDecodable {}
  impl SignedIntegerDecodable for TestDecodable {}

  #[test]
  fn decode_i32_positive() {
    assert_eq!(
      // 128
      TestDecodable::new(vec![0x80, 0x01])
        .decode_leb128_i32()
        .map(|x| x as i32),
      Ok(128)
    );
  }

  #[test]
  fn decode_i32_negative() {
    assert_eq!(
      // -128
      TestDecodable::new(vec![0x80, 0x7f])
        .decode_leb128_i32()
        .map(|x| x as i32),
      Ok(-128)
    );
  }

  #[test]
  fn decode_i32_min() {
    assert_eq!(
      // -2147483648
      TestDecodable::new(vec![0x80, 0x80, 0x80, 0x80, 0x78])
        .decode_leb128_i32()
        .map(|x| x as i32),
      Ok(std::i32::MIN)
    );
  }

  #[test]
  fn decode_i32_max() {
    assert_eq!(
      // 2147483647
      TestDecodable::new(vec![0xff, 0xff, 0xff, 0xff, 0x07])
        .decode_leb128_i32()
        .map(|x| x as i32),
      Ok(std::i32::MAX)
    );
  }

  #[test]
  fn decode_i64_min() {
    assert_eq!(
      // -9223372036854775808
      TestDecodable::new(vec![
        0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x7f,
      ])
      .decode_leb128_i64()
      .map(|x| x as i64),
      Ok(std::i64::MIN)
    );
  }

  #[test]
  fn decode_i64_max() {
    assert_eq!(
      // 9223372036854775807
      TestDecodable::new(vec![
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00,
      ])
      .decode_leb128_i64()
      .map(|x| x as i64),
      Ok(std::i64::MAX)
    );
  }
}
