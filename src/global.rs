use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use module::{
  ExternalInterface, ExternalInterfaces, ExternalModules, ImportDescriptor, ModuleDescriptor,
  ModuleDescriptorKind,
};
use trap::{Result, Trap};
use value::Values;
use value_type::ValueTypes;

#[derive(Debug, Clone)]
pub enum GlobalType {
  Const(ValueTypes),
  Var(ValueTypes),
}

impl GlobalType {
  pub fn new(code: Option<u8>, v: ValueTypes) -> Self {
    match code {
      Some(0x00) => GlobalType::Const(v),
      Some(0x01) => GlobalType::Var(v),
      x => unreachable!("Expected global type code, got {:?}", x),
    }
  }
}

#[derive(Debug)]
struct GlobalInstanceImpl {
  global_type: GlobalType,
  pub value: Values,
  pub export_name: Option<String>,
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

  fn is_same_name(&self, name: &String) -> bool {
    &self.0.borrow().export_name == &Some(name.to_owned())
  }
}

#[derive(Debug, Clone)]
pub struct GlobalInstances(Rc<RefCell<Vec<GlobalInstance>>>);

impl GlobalInstances {
  pub fn new(global_instances: Vec<GlobalInstance>) -> Self {
    GlobalInstances(Rc::new(RefCell::new(global_instances)))
  }

  pub fn empty() -> Self {
    GlobalInstances::new(vec![])
  }

  pub fn new_with_external(
    globals: Vec<(GlobalType, Values)>,
    exports: &ExternalInterfaces,
    imports: &Vec<ExternalInterface>,
    external_modules: &ExternalModules,
  ) -> Result<Self> {
    let mut global_instances: Vec<GlobalInstance> = vec![];
    for import in imports.iter() {
      match import {
        ExternalInterface {
          descriptor: ModuleDescriptor::ImportDescriptor(ImportDescriptor::Global(_)),
          name,
          module_name,
        } => {
          let global_instance = external_modules
            .find_global_instances(module_name)?
            .find(name)
            .ok_or(Trap::UnknownImport)?;
          global_instances.push(global_instance);
        }
        x => unreachable!("Expected global descriptor, got {:?}", x),
      };
    }
    for (idx, (global_type, value)) in globals.into_iter().enumerate() {
      let export_name = exports
        .find_kind_by_idx(idx as u32, ModuleDescriptorKind::Global)
        .map(|x| x.name.to_owned());
      global_instances.push(GlobalInstance::new(global_type, value, export_name));
    }
    Ok(GlobalInstances::new(global_instances))
  }

  pub fn find(&self, name: &String) -> Option<GlobalInstance> {
    self
      .0
      .borrow()
      .iter()
      .find(|instance| instance.is_same_name(name))
      .map(|x| x.clone())
  }

  pub fn get_global(&self, idx: u32) -> Result<Values> {
    self
      .0
      .borrow()
      .get(idx as usize)
      .map(|g| g.get_value().to_owned())
      .ok_or(Trap::Notfound)
  }

  pub fn get_global_ext(&self, idx: u32) -> i32 {
    self
      .get_global(idx)
      .map(|g| match g {
        Values::I32(ref v) => *v,
        x => unreachable!("Expect Values::I32, got {:?}", x),
      })
      .expect(&format!(
        "Expect to get {:?} of global instances, got None",
        idx
      ))
  }

  pub fn set_global(&self, idx: u32, value: Values) {
    self
      .0
      .borrow_mut()
      .get_mut(idx as usize)
      .map(|g| g.set_value(value));
  }
}
