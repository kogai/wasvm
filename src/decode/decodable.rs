use alloc::string::String;
use trap::Result;

#[macro_export]
macro_rules! impl_decode_leb128 {
  ($t:ty, $buf_size: ty, $fn_name: ident) => {
    #[allow(dead_code)]
    fn $fn_name(&mut self) -> $crate::trap::Result<$t> {
      let mut buf: $buf_size = 0;
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
        let is_msb_zero = raw_code & 0b10000000 == 0;
        let num = (raw_code & 0b01111111) as $buf_size; // Drop leftmost bit
        // buf =      00000000_00000000_10000000_00000000
        // num =      00000000_00000000_00000000_00000001
        // num << 7 = 00000000_00000000_00000000_10000000
        // buf | num  00000000_00000000_10000000_10000000
        let (shifted, is_overflowed) = num.overflowing_shl(shift);
        if is_overflowed {
          return Err(Trap::IntegerRepresentationTooLong);
        }
        buf |= shifted;
        shift += 7;
        if is_msb_zero {
          break;
        }
      }
      let (signed_bits, overflowed) = (1 as $buf_size).overflowing_shl(shift - 1);
      if overflowed {
        return Ok(buf as $t)
      }
      let is_buf_signed = buf & signed_bits != 0;
      if is_buf_signed {
        buf |= !0 << shift;
      };
      Ok(buf as $t)
    }
  };
}

macro_rules! impl_decode_float {
  ($ty: ty, $buf_ty: ty, $fn_name: ident, $convert: path, $bitwidth: expr) => {
    #[allow(dead_code)]
    fn $fn_name(&mut self) -> $crate::trap::Result<$ty> {
      let mut buf: $buf_ty = 0;
      let mut shift = 0;
      for _ in 0..($bitwidth / 8) {
        let num = self.next()? as $buf_ty;
        buf = buf ^ (num << shift);
        shift += 8;
      }
      Ok($convert(buf))
    }
  };
}

macro_rules! impl_decodable {
  ($name: ident) => {
    pub struct $name {
      bytes: Vec<u8>,
      byte_ptr: usize,
    }

    impl $name {
      impl_decode_leb128!(i32, u32, decode_leb128_i32);
      impl_decode_leb128!(i64, u64, decode_leb128_i64);
      impl_decode_float!(f32, u32, decode_f32, f32::from_bits, 32);
      impl_decode_float!(f64, u64, decode_f64, f64::from_bits, 64);

      // FIXME: Generalize with macro decoding signed integer.
      #[allow(dead_code)]
      fn decode_leb128_u32(&mut self) -> Result<u32> {
        let mut buf: u32 = 0;
        let mut shift = 0;
        while (self.peek()? & 0b10000000) != 0 {
          let num = (self.next()? ^ (0b10000000)) as u32;
          buf = buf ^ (num << shift);
          shift += 7;
        }
        let num = (self.next()?) as u32;
        let (shifted, is_overflowed) = num.overflowing_shl(shift);
        if is_overflowed {
          return Err(Trap::IntegerRepresentationTooLong);
        }
        buf = buf ^ shifted;
        Ok(buf)
      }

      pub fn new(bytes: Vec<u8>) -> Self {
        $name {
          bytes: bytes,
          byte_ptr: 0,
        }
      }

      fn peek(&self) -> Option<u8> {
        self.bytes.get(self.byte_ptr).map(|&x| x)
      }

      fn next(&mut self) -> Option<u8> {
        let el = self.bytes.get(self.byte_ptr);
        self.byte_ptr += 1;
        el.map(|&x| x)
      }
    }
  };
}

macro_rules! impl_decode_limit {
  ($name: ident) => {
    impl $name {
      fn decode_limit(&mut self) -> $crate::trap::Result<$crate::memory::Limit> {
        use $crate::memory::Limit::*;
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
  };
}

pub trait NameDecodable {
  fn decode_name(&mut self) -> Result<String>;
}

macro_rules! impl_name_decodable {
  ($name: ident) => {
    impl NameDecodable for $name {
      fn decode_name(&mut self) -> Result<String> {
        let size_of_name = self.decode_leb128_u32()?;
        let mut buf = vec![];
        for _ in 0..size_of_name {
          buf.push(self.next()?);
        }
        Ok(String::from_utf8(buf).expect("To encode export name has been failured."))
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
  use trap::Trap;

  impl_decodable!(TestDecodable);

  impl TestDecodable {
    impl_decode_leb128!(i8, u8, decode_leb128_i8);
  }

  #[test]
  fn decode_i32_positive() {
    assert_eq!(
      // 128
      TestDecodable::new(vec![0x80, 0x01]).decode_leb128_i32(),
      Ok(128)
    );
  }

  #[test]
  fn decode_i32_negative() {
    assert_eq!(
      // -128
      TestDecodable::new(vec![0x80, 0x7f]).decode_leb128_i32(),
      Ok(-128)
    );
  }

  #[test]
  fn decode_i32_min() {
    assert_eq!(
      // -2147483648
      TestDecodable::new(vec![0x80, 0x80, 0x80, 0x80, 0x78]).decode_leb128_i32(),
      Ok(std::i32::MIN)
    );
  }

  #[test]
  fn decode_i32_max() {
    assert_eq!(
      // 2147483647
      TestDecodable::new(vec![0xff, 0xff, 0xff, 0xff, 0x07]).decode_leb128_i32(),
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
      .decode_leb128_i64(),
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
      .decode_leb128_i64(),
      Ok(std::i64::MAX)
    );
  }
}
