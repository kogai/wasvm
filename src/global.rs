#[cfg(not(test))]
use alloc::prelude::*;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use indice::Indice;
use isa::Code;
use module::{
  ExternalInterface, ExternalInterfaces, ExternalModules, ImportDescriptor, ModuleDescriptor,
  GLOBAL_DESCRIPTOR,
};
use trap::{Result, Trap};
use value::Values;
use value_type::ValueTypes;

#[derive(Debug, Clone, PartialEq)]
pub enum GlobalType {
  Const(ValueTypes),
  Var(ValueTypes),
}

impl GlobalType {
  pub fn new(code: Option<u8>, v: ValueTypes) -> Result<Self> {
    match code {
      Some(0x00) => Ok(GlobalType::Const(v)),
      Some(0x01) => Ok(GlobalType::Var(v)),
      _ => Err(Trap::InvalidMutability),
    }
  }
}

#[derive(Debug)]
struct GlobalInstanceImpl {
  global_type: GlobalType,
  value: Values,
  export_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GlobalInstance(Rc<RefCell<GlobalInstanceImpl>>);

impl GlobalInstance {
  pub fn new(global_type: GlobalType, value: Values, export_name: Option<String>) -> Self {
    GlobalInstance(Rc::new(RefCell::new(GlobalInstanceImpl {
      global_type,
      value,
      export_name,
    })))
  }

  pub fn get_value(&self) -> Values {
    self.0.borrow().value.clone()
  }

  pub fn set_value(&self, value: Values) {
    self.0.borrow_mut().value = value;
  }

  fn is_same_name(&self, name: &str) -> bool {
    self.0.borrow().export_name == Some(name.to_string())
  }

  fn is_same_type(&self, ty: &GlobalType) -> bool {
    &self.0.borrow().global_type == ty
  }
}

#[derive(Debug, Clone)]
pub struct GlobalInstances(Rc<RefCell<Vec<GlobalInstance>>>);

impl GlobalInstances {
  pub fn new(global_instances: Vec<GlobalInstance>) -> Self {
    GlobalInstances(Rc::new(RefCell::new(global_instances)))
  }

  // TODO: Use Default trait.
  pub fn empty() -> Self {
    GlobalInstances::new(vec![])
  }

  pub fn new_with_external(
    globals: Vec<(GlobalType, Vec<u8>)>,
    exports: &ExternalInterfaces,
    imports: &[ExternalInterface],
    external_modules: &ExternalModules,
  ) -> Result<Self> {
    let mut global_instances: Vec<GlobalInstance> = vec![];
    for import in imports.iter() {
      match import {
        ExternalInterface {
          descriptor: ModuleDescriptor::ImportDescriptor(ImportDescriptor::Global(ty)),
          name,
          module_name,
        } => {
          let global_instance = external_modules
            .find_global_instances(module_name)?
            .find(name)
            .ok_or(Trap::UnknownImport)?;
          if !global_instance.is_same_type(ty) {
            return Err(Trap::IncompatibleImportType);
          }
          global_instances.push(global_instance);
        }
        x => unreachable!("Expected global descriptor, got {:?}", x),
      };
    }
    for (idx, (global_type, init)) in globals.into_iter().enumerate() {
      let export_name = exports
        .find_kind_by_idx(idx as u32, &GLOBAL_DESCRIPTOR)
        .map(|x| x.name.to_owned());
      let init_first = init.first()?;
      let value = match Code::from(*init_first) {
        Code::ConstI32 => {
          let mut buf = [0; 4];
          buf.clone_from_slice(&init[1..5]);
          Values::I32(unsafe { core::mem::transmute::<_, u32>(buf) } as i32)
        }
        Code::ConstI64 => {
          let mut buf = [0; 8];
          buf.clone_from_slice(&init[1..9]);
          Values::I64(unsafe { core::mem::transmute::<_, u64>(buf) } as i64)
        }
        Code::F32Const => {
          let mut buf = [0; 4];
          buf.clone_from_slice(&init[1..5]);
          Values::F32(f32::from_bits(unsafe {
            core::mem::transmute::<_, u32>(buf)
          }))
        }
        Code::F64Const => {
          let mut buf = [0; 8];
          buf.clone_from_slice(&init[1..9]);
          Values::F64(f64::from_bits(unsafe {
            core::mem::transmute::<_, u64>(buf)
          }))
        }
        Code::GetGlobal => {
          let mut buf = [0; 4];
          buf.clone_from_slice(&init[1..5]);
          let idx = Indice::from(unsafe { core::mem::transmute::<_, u32>(buf) });
          global_instances.get(idx.to_usize())?.get_value()
        }
        x => unreachable!("Expected initial value of global, got {:?}", x),
      };
      global_instances.push(GlobalInstance::new(global_type, value, export_name));
    }
    Ok(GlobalInstances::new(global_instances))
  }

  pub fn find(&self, name: &str) -> Option<GlobalInstance> {
    self
      .0
      .borrow()
      .iter()
      .find(|instance| instance.is_same_name(name))
      .cloned()
  }

  pub fn get_global(&self, idx: &Indice) -> Result<Values> {
    self
      .0
      .borrow()
      .get(idx.to_usize())
      .map(|g| g.get_value().to_owned())
      .ok_or(Trap::Notfound)
  }

  pub fn get_global_ext(&self, idx: &Indice) -> i32 {
    self
      .get_global(idx)
      .map(|g| match g {
        Values::I32(ref v) => *v,
        x => unreachable!("Expect Values::I32, got {:?}", x),
      })
      .unwrap_or_else(|_| panic!("Expect to get {:?} of global instances, got None", idx))
  }

  pub fn set_global(&self, idx: &Indice, value: Values) {
    if let Some(g) = self.0.borrow_mut().get_mut(idx.to_usize()) {
      g.set_value(value)
    };
  }
}
