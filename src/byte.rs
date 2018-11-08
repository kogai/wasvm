#[derive(Debug, PartialEq, Clone)]
pub enum ValueTypes {
  I32,
  // I64,
  // F32,
  // F64,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Values {
  I32(i32),
  // I64,
  // F32,
  // F64,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Code {
  SectionType,
  SectionFunction,
  SectionExport,
  SectionCode,
  ConstI32,
  Value(Values),
  ValueType(ValueTypes),
  TypeFunction,

  IdxOfType(u8),
  IdxOfFunction(u8),
  ExportName(String),

  ExportDescFunctionIdx,
  ExportDescTableIdx,
  ExportDescMemIdx,
  ExportDescGlobalIdx,

  End,
}

impl Code {
  fn from_byte(code: Option<u8>) -> Self {
    use self::Code::*;

    match code {
      Some(0x1) => SectionType,
      Some(0x3) => SectionFunction,
      Some(0x7) => SectionExport,
      Some(0xa) => SectionCode,
      Some(0x7f) => ValueType(ValueTypes::I32),
      Some(0x41) => ConstI32,
      Some(0x60) => TypeFunction,
      Some(0x0b) => End,
      _ => unreachable!(),
    }
  }

  fn from_byte_to_export_description(code: Option<u8>) -> Self {
    use self::Code::*;
    match code {
      Some(0x00) => ExportDescFunctionIdx,
      Some(0x01) => ExportDescTableIdx,
      Some(0x02) => ExportDescMemIdx,
      Some(0x03) => ExportDescGlobalIdx,
      _ => unreachable!(),
    }
  }
}

#[derive(Debug, PartialEq)]
pub struct Byte {
  bytes: Vec<u8>,
  pub bytes_decoded: Vec<Code>,
  byte_ptr: usize,
}

impl Byte {
  pub fn new(bytes: Vec<u8>) -> Self {
    Byte {
      bytes,
      bytes_decoded: vec![],
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

  pub fn decode(&mut self) -> Option<()> {
    while self.has_next() {
      let section_code = Code::from_byte(self.next());
      let mut internal_section = vec![];

      match &section_code {
        Code::SectionType => {
          let _size_of_section = self.next()?;
          let size_of_type = self.next()?;
          for _ in 0..size_of_type {
            let type_function = Code::from_byte(self.next());
            internal_section.push(type_function);
            let size_of_arity = self.next()?;
            for _ in 0..size_of_arity {
              internal_section.push(Code::from_byte(self.next()));
            }
            let size_of_result = self.next()?;
            for _ in 0..size_of_result {
              internal_section.push(Code::from_byte(self.next()));
            }
          }
        }
        Code::SectionFunction => {
          let _size_of_section = self.next()?;
          let size_of_type_idx = self.next()?;
          for _ in 0..size_of_type_idx {
            internal_section.push(Code::IdxOfType(self.next()?));
          }
        }
        Code::SectionExport => {
          let _size_of_section = self.next()?;
          let size_of_export = self.next()?;
          for _ in 0..size_of_export {
            let size_of_name = self.next().unwrap();
            let mut buf = vec![];
            for _ in 0..size_of_name {
              buf.push(self.next()?);
            }
            internal_section.push(Code::ExportName(
              String::from_utf8(buf).expect("To encode export name has been failured."),
            ));
            match Code::from_byte_to_export_description(self.next()) {
              Code::ExportDescFunctionIdx => {
                internal_section.push(Code::IdxOfFunction(self.next()?));
              }
              _ => unimplemented!(),
            }
          }
        }
        Code::SectionCode => {
          let _size_of_section = self.next()?;
          let size_of_code = self.next()?;
          for _ in 0..size_of_code {
            let _size_of_function = self.next()?;
            let size_of_locals = self.next()?;
            for _ in 0..size_of_locals {
              unimplemented!();
            }
            while !(Code::from_byte(self.peek()) == Code::End) {
              let operation = Code::from_byte(self.next());
              let expressions = match operation {
                Code::ConstI32 => Code::Value(Values::I32(self.next()? as i32)),
                _ => unimplemented!(),
              };
              internal_section.push(operation);
              internal_section.push(expressions);
            }
            self.next(); // Drop End code.
          }
        }
        x => {
          println!("{:?}", x);
          unreachable!();
        }
      };

      self.bytes_decoded.push(section_code);
      self.bytes_decoded.append(&mut internal_section);
    }
    Some(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use utils::read_wasm;

  #[test]
  fn it_decode() {
    use self::Code::*;
    let wasm = read_wasm("./dist/constant.wasm").unwrap();
    let mut bc = Byte::new(wasm);
    bc.decode();
    assert_eq!(
      bc.bytes_decoded,
      vec![
        SectionType,
        TypeFunction,
        ValueType(ValueTypes::I32),
        SectionFunction,
        IdxOfType(0),
        SectionExport,
        ExportName("_subject".to_owned()),
        IdxOfFunction(0),
        SectionCode,
        ConstI32,
        Value(Values::I32(42))
      ]
    );
  }
}
