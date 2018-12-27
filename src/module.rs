use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ModuleDescriptor {
  Function(u32),
  Table(u32),
  Memory(u32),
  Global(u32),
}

impl From<(Option<u8>, u32)> for ModuleDescriptor {
  fn from(codes: (Option<u8>, u32)) -> Self {
    use self::ModuleDescriptor::*;
    match codes.0 {
      Some(0x0) => Function(codes.1),
      Some(0x1) => Table(codes.1),
      Some(0x2) => Memory(codes.1),
      Some(0x3) => Global(codes.1),
      x => unreachable!("Expected import descriptor, got {:?}", x),
    }
  }
}

#[derive(Debug, Clone)]
pub struct ExternalInterface {
  module_name: Option<String>,
  pub name: String,
  pub descriptor: ModuleDescriptor,
}

impl ExternalInterface {
  pub fn new(module_name: Option<String>, name: String, descriptor: ModuleDescriptor) -> Self {
    ExternalInterface {
      module_name,
      name,
      descriptor,
    }
  }
}

#[derive(Debug)]
pub struct ExternalInterfaces(HashMap<String, ExternalInterface>);

impl ExternalInterfaces {
  pub fn new() -> Self {
    ExternalInterfaces(HashMap::new())
  }

  pub fn insert(&mut self, key: String, value: ExternalInterface) {
    self.0.insert(key, value);
  }

  pub fn find_by_idx(&self, idx: u32) -> Option<&ExternalInterface> {
    self
      .0
      .iter()
      .find(
        |(_key, ExternalInterface { descriptor, .. })| match descriptor {
          ModuleDescriptor::Function(x)
          | ModuleDescriptor::Table(x)
          | ModuleDescriptor::Memory(x)
          | ModuleDescriptor::Global(x) => *x == idx,
        },
      )
      .map(|(_, x)| x)
  }
}

pub struct InternalModule {
  exports: ExternalInterfaces,
  imports: ExternalInterfaces,
}

impl InternalModule {
  pub fn new(exports: ExternalInterfaces, imports: ExternalInterfaces) -> Self {
    InternalModule { exports, imports }
  }

  pub fn get_export_by_key(&self, invoke: &str) -> Option<&ExternalInterface> {
    self.exports.0.get(invoke)
  }
}
