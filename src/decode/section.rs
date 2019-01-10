use super::context::Context;
use super::sec_element::Element;
use super::sec_table::TableType;
use super::Data;
use alloc::prelude::*;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::default::Default;
use function::{FunctionInstance, FunctionType};
use global::{GlobalInstances, GlobalType};
use inst::Inst;
use memory::{Limit, MemoryInstance};
use module::{
  ExternalInterface, ExternalInterfaces, ExternalModules, InternalModule, ModuleDescriptorKind,
};
use store::Store;
use table::{TableInstance, TableInstances};
use trap::{Result, Trap};
use value::Values;
use value_type::ValueTypes;

#[derive(Debug, PartialEq, Clone)]
pub enum SectionCode {
  Custom,
  Type,
  Import,
  Function,
  Table,
  Memory,
  Global,
  Export,
  Start,
  Element,
  Code,
  Data,
}

impl TryFrom<Option<u8>> for SectionCode {
  type Error = Trap;
  fn try_from(code: Option<u8>) -> core::result::Result<Self, Self::Error> {
    use self::SectionCode::*;
    match code {
      Some(0x0) => Ok(Custom),
      Some(0x1) => Ok(Type),
      Some(0x2) => Ok(Import),
      Some(0x3) => Ok(Function),
      Some(0x4) => Ok(Table),
      Some(0x5) => Ok(Memory),
      Some(0x6) => Ok(Global),
      Some(0x7) => Ok(Export),
      Some(0x8) => Ok(Start),
      Some(0x9) => Ok(Element),
      Some(0xa) => Ok(Code),
      Some(0xb) => Ok(Data),
      _ => Err(Trap::InvalidSectionId),
    }
  }
}

#[derive(Debug)]
pub struct Section {
  function_types: Vec<FunctionType>,
  functions: Vec<u32>,
  exports: ExternalInterfaces,
  codes: Vec<Result<(Vec<Inst>, Vec<ValueTypes>)>>,
  datas: Vec<Data>,
  limits: Vec<Limit>,
  tables: Vec<TableType>,
  globals: Vec<(GlobalType, Values)>,
  elements: Vec<Element>,
  customs: Vec<(String, Vec<u8>)>,
  imports: ExternalInterfaces,
  start: Option<u32>,
}

impl Default for Section {
  fn default() -> Section {
    Section {
      function_types: vec![],
      functions: vec![],
      exports: ExternalInterfaces::new(),
      codes: vec![],
      datas: vec![],
      limits: vec![],
      tables: vec![],
      globals: vec![],
      elements: vec![],
      customs: vec![],
      imports: ExternalInterfaces::new(),
      start: None,
    }
  }
}

macro_rules! impl_builder {
  ($name: ident, $prop: ident, $ty: ty) => {
    pub fn $name<'a>(&'a mut self, xs: &mut Vec<$ty>) -> &'a mut Self {
      self.$prop.append(xs);
      self
    }
  };
}

impl Section {
  impl_builder!(function_types, function_types, FunctionType);
  impl_builder!(functions, functions, u32);
  impl_builder!(codes, codes, Result<(Vec<Inst>, Vec<ValueTypes>)>);
  impl_builder!(datas, datas, Data);
  impl_builder!(limits, limits, Limit);
  impl_builder!(tables, tables, TableType);
  impl_builder!(globals, globals, (GlobalType, Values));
  impl_builder!(elements, elements, Element);
  impl_builder!(customs, customs, (String, Vec<u8>));

  pub fn imports<'a>(&'a mut self, xs: ExternalInterfaces) -> &'a mut Self {
    self.imports = xs;
    self
  }

  pub fn exports<'a>(&'a mut self, xs: ExternalInterfaces) -> &'a mut Self {
    self.exports = xs;
    self
  }

  pub fn start<'a>(&'a mut self, x: u32) -> &'a mut Self {
    self.start = Some(x);
    self
  }

  fn memory_instances(
    datas: Vec<Data>,
    limits: Vec<Limit>,
    exports: &ExternalInterfaces,
    imports: &Vec<ExternalInterface>,
    external_modules: &ExternalModules,
    global_instances: &GlobalInstances,
  ) -> Result<Vec<MemoryInstance>> {
    // NOTE: Currently WASM specification assumed only one memory instance;
    let memory_idx = 0;
    let export_name = exports
      .find_kind_by_idx(memory_idx, ModuleDescriptorKind::Memory)
      .map(|x| x.name.to_owned());

    let external_memory_types = imports
      .iter()
      .map(|value| external_modules.find_memory_instance(value))
      .collect::<Result<Vec<MemoryInstance>>>()?;

    if let Some(limit) = limits.get(memory_idx as usize) {
      Ok(vec![MemoryInstance::new(
        datas,
        limit.to_owned(),
        export_name,
        global_instances,
      )?])
    } else if let Some(memory_instance) = external_memory_types.get(memory_idx as usize) {
      Ok(vec![MemoryInstance::new(
        datas,
        memory_instance.limit.to_owned(),
        export_name,
        global_instances,
      )?])
    } else {
      Ok(vec![])
    }
  }

  fn table_instances(
    elements: Vec<Element>,
    tables: Vec<TableType>,
    exports: &ExternalInterfaces,
    imports: &Vec<ExternalInterface>,
    external_modules: &ExternalModules,
    global_instances: &GlobalInstances,
    function_instances: &Vec<Rc<FunctionInstance>>,
  ) -> Result<TableInstances> {
    if tables.len() > 0 {
      tables
        .into_iter()
        .map(|table_type| {
          let export_name = exports
            .find_kind_by_idx(0, ModuleDescriptorKind::Table)
            .map(|x| x.name.to_owned());
          TableInstance::new(
            elements.clone(),
            table_type,
            export_name,
            global_instances,
            function_instances,
          )
        })
        .collect::<Result<Vec<_>>>()
        .map(|table_instances| TableInstances::new(table_instances))
    } else {
      // NOTE: Only one table instance allowed.
      match imports.first() {
        Some(import) => external_modules
          .find_table_instance(import)
          .map(|table_instances| {
            table_instances.update(elements.clone(), global_instances, function_instances)?;
            Ok(table_instances)
          })?,
        None => Ok(TableInstances::empty()),
      }
    }
  }

  fn function_type(idx: usize, function_types: &Vec<FunctionType>) -> FunctionType {
    function_types
      .get(idx)
      .expect("Function type can't found.")
      .to_owned()
  }

  fn function_instances(
    function_types: &Vec<FunctionType>,
    functions: Vec<u32>,
    exports: &ExternalInterfaces,
    codes: Vec<Result<(Vec<Inst>, Vec<ValueTypes>)>>,
  ) -> Result<Vec<Rc<FunctionInstance>>> {
    codes
      .into_iter()
      .enumerate()
      .map(|(idx, code)| {
        let export_name = exports
          .find_kind_by_idx(idx as u32, ModuleDescriptorKind::Function)
          .map(|x| x.name.to_owned());
        let index_of_type = match functions.get(idx) {
          Some(n) => *n,
          None => return Err(Trap::FunctionAndCodeInconsitent),
        };
        let function_type = Section::function_type(index_of_type as usize, function_types);
        let (expressions, locals) = code?;
        Ok(FunctionInstance::new(
          export_name,
          function_type,
          locals,
          expressions,
        ))
      })
      .collect::<Result<Vec<_>>>()
  }

  fn external_function_instances(
    function_types: &Vec<FunctionType>,
    imports: &Vec<ExternalInterface>,
    external_modules: &ExternalModules,
  ) -> Result<Vec<Rc<FunctionInstance>>> {
    imports
      .iter()
      .map(|value| external_modules.find_function_instances(value, function_types))
      .collect::<Result<Vec<_>>>()
  }

  pub fn complete(self, external_modules: ExternalModules) -> Result<(Store, InternalModule)> {
    match self {
      Section {
        function_types,
        functions,
        codes,
        exports,
        datas,
        limits,
        tables,
        elements,
        globals,
        customs: _,
        imports,
        start,
      } => {
        let grouped_imports = imports.group_by_kind();
        let imports_function = grouped_imports.get(&ModuleDescriptorKind::Function)?;
        let imports_table = grouped_imports.get(&ModuleDescriptorKind::Table)?;
        let imports_memory = grouped_imports.get(&ModuleDescriptorKind::Memory)?;
        let imports_global = grouped_imports.get(&ModuleDescriptorKind::Global)?;

        let mut internal_function_instances =
          Section::function_instances(&function_types, functions, &exports, codes)?;

        let mut function_instances = Section::external_function_instances(
          &function_types,
          &imports_function,
          &external_modules,
        )?;

        function_instances.append(&mut internal_function_instances);

        let global_instances = GlobalInstances::new_with_external(
          globals,
          &exports,
          &imports_global,
          &external_modules,
        )?;

        let memory_instances = Section::memory_instances(
          datas,
          limits,
          &exports,
          &imports_memory,
          &external_modules,
          &global_instances,
        )?;
        let mut table_instances = Section::table_instances(
          elements,
          tables,
          &exports,
          &imports_table,
          &external_modules,
          &global_instances,
          &function_instances,
        )?;

        Ok(
          Context::new(
            function_instances,
            function_types,
            memory_instances,
            table_instances,
            global_instances,
            exports,
            imports,
            start,
          )
          .without_validate()?,
        )
      }
    }
  }
}
