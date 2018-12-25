use trap::Result;

#[macro_export]
macro_rules! impl_decode_leb128 {
  ($t:ty, $buf_size: ty, $fn_name: ident) => {
    #[allow(dead_code)]
    fn $fn_name(&mut self) -> $crate::trap::Result<$t> {
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
        return Err($crate::trap::Trap::BitshiftOverflow)
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
            let min = self.decode_leb128_i32()?;
            Ok(NoUpperLimit(min as u32))
          }
          Some(0x1) => {
            let min = self.decode_leb128_i32()?;
            let max = self.decode_leb128_i32()?;
            Ok(HasUpperLimit(min as u32, max as u32))
          }
          x => unreachable!("Expected limit code, got {:?}", x),
        }
      }
    }
  };
}

pub trait Decodable {
  type Item;
  fn decode(&mut self) -> Result<Self::Item>;
}
