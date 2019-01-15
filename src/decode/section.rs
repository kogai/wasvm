use super::sec_element::Element;
use super::sec_table::TableType;
use super::Data;
use alloc::prelude::*;
use alloc::string::String;
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::default::Default;
use function::{FunctionInstance, FunctionType};
use global::{GlobalInstances, GlobalType};
use inst::Inst;
use memory::{Limit, MemoryInstance, MemoryInstances};
use module::{
  ExternalInterface, ExternalInterfaces, ExternalModules, InternalModule, FUNCTION_DESCRIPTOR,
  GLOBAL_DESCRIPTOR, MEMORY_DESCRIPTOR, TABLE_DESCRIPTOR,
};
use store::Store;
use table::{TableInstance, TableInstances};
use trap::{Result, Trap};
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

// FIXME: Rename to `Module`
#[derive(Debug)]
pub struct Section {
  pub(crate) function_types: Vec<FunctionType>,
  pub(crate) functions: Vec<u32>,
  pub(crate) exports: ExternalInterfaces,
  pub(crate) codes: Vec<Result<(Vec<Inst>, Vec<ValueTypes>)>>,
  pub(crate) datas: Vec<Data>,
  pub(crate) limits: Vec<Limit>,
  pub(crate) tables: Vec<TableType>,
  pub(crate) globals: Vec<(GlobalType, Vec<Inst>)>,
  pub(crate) elements: Vec<Element>,
  pub(crate) customs: Vec<(String, Vec<u8>)>,
  pub(crate) imports: ExternalInterfaces,
  pub(crate) start: Option<u32>,
}

impl Default for Section {
  fn default() -> Section {
    Section {
      function_types: vec![],
      functions: vec![],
      exports: ExternalInterfaces::default(),
      codes: vec![],
      datas: vec![],
      limits: vec![],
      tables: vec![],
      globals: vec![],
      elements: vec![],
      customs: vec![],
      imports: ExternalInterfaces::default(),
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
  impl_builder!(globals, globals, (GlobalType, Vec<Inst>));
  impl_builder!(elements, elements, Element);
  impl_builder!(customs, customs, (String, Vec<u8>));

  pub fn imports(&mut self, xs: ExternalInterfaces) -> &mut Self {
    self.imports = xs;
    self
  }

  pub fn exports(&mut self, xs: ExternalInterfaces) -> &mut Self {
    self.exports = xs;
    self
  }

  pub fn start(&mut self, x: u32) -> &mut Self {
    self.start = Some(x);
    self
  }

  fn validate_memory(
    datas: &[Data],
    limits: &[Limit],
    imports: &[ExternalInterface],
    external_modules: &ExternalModules,
    global_instances: &GlobalInstances,
  ) -> Result<()> {
    let memory_idx = 0;
    let external_memory_instances = imports
      .get(memory_idx as usize)
      .map(|key| external_modules.find_memory_instances(key));

    if limits.get(memory_idx as usize).is_some() && external_memory_instances.is_none() {
      return Ok(());
    }
    if external_memory_instances.is_some() {
      MemoryInstances::validate(
        &external_memory_instances??,
        &limits
          .get(memory_idx as usize)
          .map(|limit| limit.to_owned()),
        imports.get(memory_idx as usize)?,
        datas,
        global_instances,
      )
    } else {
      Ok(())
    }
  }

  fn validate_table(
    elements: &[Element],
    tables: &[TableType],
    imports: &[ExternalInterface],
    external_modules: &ExternalModules,
    global_instances: &GlobalInstances,
    function_instances: &[FunctionInstance],
  ) -> Result<()> {
    if !tables.is_empty() {
      tables
        .iter()
        .map(|table_type| {
          TableInstance::validate(elements, table_type, global_instances, function_instances)
        })
        .collect::<Result<Vec<_>>>()
        .and_then(|_| Ok(()))
    } else {
      match imports.first() {
        Some(import) => external_modules
          .find_table_instances(import)
          .and_then(|table_instances| {
            table_instances.validate(elements, global_instances, function_instances)
          }),
        None => Ok(()),
      }
    }
  }

  fn memory_instances(
    datas: Vec<Data>,
    limits: &[Limit],
    exports: &ExternalInterfaces,
    imports: &[ExternalInterface],
    external_modules: &ExternalModules,
    global_instances: &GlobalInstances,
  ) -> Result<MemoryInstances> {
    // NOTE: Currently WASM specification assumed only one memory instance;
    let memory_idx = 0;
    let export_name = exports
      .find_kind_by_idx(memory_idx, &MEMORY_DESCRIPTOR)
      .map(|x| x.name.to_owned());

    let external_memory_instances = imports
      .get(memory_idx as usize)
      .map(|key| external_modules.find_memory_instances(key));

    if let Some(limit) = limits.get(memory_idx as usize) {
      if external_memory_instances.is_none() {
        return Ok(MemoryInstances::new(vec![MemoryInstance::new(
          datas,
          limit.to_owned(),
          export_name,
          global_instances,
        )?]));
      }
    }
    if external_memory_instances.is_some() {
      MemoryInstances::from(
        &external_memory_instances??,
        limits
          .get(memory_idx as usize)
          .map(|limit| limit.to_owned()),
        datas,
        global_instances,
      )
    } else {
      Ok(MemoryInstances::empty())
    }
  }

  fn table_instances(
    elements: &[Element],
    tables: Vec<TableType>,
    exports: &ExternalInterfaces,
    imports: &[ExternalInterface],
    external_modules: &ExternalModules,
    global_instances: &GlobalInstances,
    function_instances: &[FunctionInstance],
  ) -> Result<TableInstances> {
    if !tables.is_empty() {
      tables
        .into_iter()
        .map(|table_type| {
          let export_name = exports
            .find_kind_by_idx(0, &TABLE_DESCRIPTOR)
            .map(|x| x.name.to_owned());
          TableInstance::new(
            elements.to_vec(),
            table_type,
            export_name,
            global_instances,
            function_instances,
          )
        })
        .collect::<Result<Vec<_>>>()
        .map(TableInstances::new)
    } else {
      // NOTE: Only one table instance allowed.
      match imports.first() {
        Some(import) => external_modules
          .find_table_instances(import)
          .map(|table_instances| {
            table_instances.link(&elements, global_instances, function_instances)?;
            Ok(table_instances)
          })?,
        None => Ok(TableInstances::empty()),
      }
    }
  }

  fn function_type(idx: usize, function_types: &[FunctionType]) -> FunctionType {
    function_types
      .get(idx)
      .expect("Function type can't found.")
      .to_owned()
  }

  fn function_instances(
    function_types: &[FunctionType],
    functions: &[u32],
    exports: &ExternalInterfaces,
    codes: Vec<Result<(Vec<Inst>, Vec<ValueTypes>)>>,
  ) -> Result<Vec<FunctionInstance>> {
    codes
      .into_iter()
      .enumerate()
      .map(|(idx, code)| {
        let export_name = exports
          .find_kind_by_idx(idx as u32, &FUNCTION_DESCRIPTOR)
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
    function_types: &[FunctionType],
    imports: &[ExternalInterface],
    external_modules: &ExternalModules,
  ) -> Result<Vec<FunctionInstance>> {
    imports
      .iter()
      .map(|value| external_modules.find_function_instances(value, function_types))
      .collect::<Result<Vec<_>>>()
  }

  pub fn complete(
    self,
    external_modules: &ExternalModules,
    store: &mut Store,
  ) -> Result<InternalModule> {
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
        imports,
        start,
        ..
      } => {
        let grouped_imports = imports.group_by_kind();
        let imports_function = grouped_imports.get(&FUNCTION_DESCRIPTOR)?;
        let imports_table = grouped_imports.get(&TABLE_DESCRIPTOR)?;
        let imports_memory = grouped_imports.get(&MEMORY_DESCRIPTOR)?;
        let imports_global = grouped_imports.get(&GLOBAL_DESCRIPTOR)?;

        let mut internal_function_instances =
          Section::function_instances(&function_types, &functions, &exports, codes)?;

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

        // TODO: Move to context mod.
        let (validate_memory, validate_table) = (
          Section::validate_memory(
            &datas,
            &limits,
            &imports_memory,
            &external_modules,
            &global_instances,
          ),
          Section::validate_table(
            &elements,
            &tables,
            &imports_table,
            &external_modules,
            &global_instances,
            &function_instances,
          ),
        );
        validate_memory?;
        validate_table?;

        let memory_instances = Section::memory_instances(
          datas,
          &limits,
          &exports,
          &imports_memory,
          &external_modules,
          &global_instances,
        )?;

        let mut table_instances = Section::table_instances(
          &elements,
          tables,
          &exports,
          &imports_table,
          &external_modules,
          &global_instances,
          &function_instances,
        )?;

        store.function_instances = function_instances;
        store.function_types = function_types;
        store.memory_instances = memory_instances;
        store.table_instances = table_instances;
        store.global_instances = global_instances;
        let internal_module = InternalModule::new(exports, start);
        Ok(internal_module)
      }
    }
  }
}
