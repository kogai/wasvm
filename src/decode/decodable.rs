// use code::{Code, ValueTypes};
// use function::FunctionType;
// use std::convert::From;
// use std::default::Default;
// use std::{f32, f64};
// use store::Store;
use trap::Result;

#[macro_export]
macro_rules! impl_decode_leb128 {
  ($t:ty, $buf_size: ty, $fn_name: ident) => {
    fn $fn_name(&mut self) -> Result<$t> {
      let mut buf: $t = 0;
      let mut shift = 0;

      // Check whether leftmost bit is 1 or 0
      // n     = 0b11111111 = 0b01111111
      // _     = 0b10000000 = 0b10000000
      // n & _ = 0b10000000 = 0b00000000
      while (self.peek()? & 0b10000000) != 0 {
        let num = (self.next()? ^ (0b10000000)) as $t; // If leftmost bit is 1, we drop it.

        // buf =      00000000_00000000_10000000_00000000
        // num =      00000000_00000000_00000000_00000001
        // num << 7 = 00000000_00000000_00000000_10000000
        // buf ^ num  00000000_00000000_10000000_10000000
        buf = buf ^ (num << shift);
        shift += 7;
      }
      let num = (self.next()?) as $t;
      buf = buf ^ (num << shift);

      let (msb_one, overflowed) = (1 as $buf_size).overflowing_shl(shift + 6);
      if overflowed {
        return Err(Trap::BitshiftOverflow)
      }
      if buf & (msb_one as $t) != 0 {
        Ok(-((1 << (shift + 7)) - buf))
      } else {
        Ok(buf)
      }
    }
  };
}

macro_rules! impl_decode_float {
  ($ty: ty, $buf_ty: ty, $fn_name: ident, $convert: path, $bitwidth: expr) => {
    fn $fn_name(&mut self) -> Result<$ty> {
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
      fn decode_leb128_u32(&mut self) -> Result<u32> {
        let mut buf: u32 = 0;
        let mut shift = 0;
        while (self.peek()? & 0b10000000) != 0 {
          let num = (self.next()? ^ (0b10000000)) as u32;
          buf = buf ^ (num << shift);
          shift += 7;
        }
        let num = (self.next()?) as u32;
        buf = buf ^ (num << shift);
        Ok(buf)
      }
      pub fn new(bytes: Vec<u8>) -> Self {
        $name {
          bytes: bytes,
          byte_ptr: 0,
        }
      }

      fn has_next(&self) -> bool {
        self.byte_ptr < self.bytes.len()
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

pub trait Decodable<T> {
  fn decode(&mut self) -> Result<Vec<T>>;
}
