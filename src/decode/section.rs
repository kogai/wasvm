use super::context::Context;
use super::sec_element::Element;
use super::sec_export::Exports;
use super::sec_table::{TableInstance, TableType};
use super::Data;
use function::{FunctionInstance, FunctionType};
use global::GlobalInstance;
use inst::Inst;
use internal_module::InternalModule;
use memory::{Limit, MemoryInstance};
use std::collections::HashMap;
use std::convert::From;
use std::default::Default;
use std::rc::Rc;
use store::Store;
use trap::Result;
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

impl From<Option<u8>> for SectionCode {
  fn from(code: Option<u8>) -> Self {
    use self::SectionCode::*;
    match code {
      Some(0x0) => Custom,
      Some(0x1) => Type,
      Some(0x2) => Import,
      Some(0x3) => Function,
      Some(0x4) => Table,
      Some(0x5) => Memory,
      Some(0x6) => Global,
      Some(0x7) => Export,
      Some(0x8) => Start,
      Some(0x9) => Element,
      Some(0xa) => Code,
      Some(0xb) => Data,
      x => unreachable!("Expect section code, got {:x?}.", x),
    }
  }
}

#[derive(Debug)]
pub struct Section {
  function_types: Vec<FunctionType>,
  functions: Vec<u32>,
  exports: Exports,
  codes: Vec<Result<(Vec<Inst>, Vec<ValueTypes>)>>,
  datas: Vec<Data>,
  limits: Vec<Limit>,
  tables: Vec<TableType>,
  globals: Vec<GlobalInstance>,
  elements: Vec<Element>,
}

impl Default for Section {
  fn default() -> Section {
    Section {
      function_types: vec![],
      functions: vec![],
      exports: HashMap::new(),
      codes: vec![],
      datas: vec![],
      limits: vec![],
      tables: vec![],
      globals: vec![],
      elements: vec![],
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
  impl_builder!(globals, globals, GlobalInstance);
  impl_builder!(elements, elements, Element);

  pub fn exports<'a>(&'a mut self, xs: Exports) -> &'a mut Self {
    self.exports = xs;
    self
  }

  fn memory_instances(datas: Vec<Data>, limits: Vec<Limit>) -> Vec<MemoryInstance> {
    if datas.is_empty() && limits.is_empty() {
      return vec![];
    };
    if datas.is_empty() {
      return vec![MemoryInstance::new(Data::new(0, vec![], vec![]), &limits)];
    };
    datas
      .into_iter()
      .map(|d| MemoryInstance::new(d, &limits))
      .collect::<Vec<_>>()
  }

  fn table_instances(elements: Vec<Element>, tables: Vec<TableType>) -> Vec<TableInstance> {
    elements
      .into_iter()
      .map(|el| {
        let table_type = tables
          .get(el.get_table_idx())
          .expect("Table type not found.");
        TableInstance::new(&table_type, el)
      })
      .collect::<Vec<_>>()
  }

  fn export_name(idx: usize, exports: &Exports) -> Option<String> {
    exports
      .iter()
      .find(|(_key, (_kind, i))| *i == idx)
      .map(|(key, _)| key.to_owned())
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
    exports: &Exports,
    codes: Vec<Result<(Vec<Inst>, Vec<ValueTypes>)>>,
  ) -> Result<Vec<Rc<FunctionInstance>>> {
    codes
      .into_iter()
      .enumerate()
      .map(|(idx, code)| {
        let export_name = Section::export_name(idx, &exports);
        let index_of_type = *functions.get(idx).expect("Index of type can't found.");
        let function_type = Section::function_type(index_of_type as usize, function_types);
        let (expressions, locals) = code?;
        Ok(FunctionInstance::new(
          export_name,
          function_type,
          locals,
          index_of_type,
          expressions,
        ))
      })
      .collect::<Result<Vec<_>>>()
  }

  // NOTE: Might be reasonable some future.
  fn global_instances(globals: Vec<GlobalInstance>) -> Vec<GlobalInstance> {
    globals
  }

  pub fn complete(self) -> Result<(Store, InternalModule)> {
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
      } => {
        let memory_instances = Section::memory_instances(datas, limits);
        let table_instances = Section::table_instances(elements, tables);
        let function_instances =
          Section::function_instances(&function_types, functions, &exports, codes)?;
        let global_instances = Section::global_instances(globals);
        Ok(
          Context::new(
            function_instances,
            function_types,
            memory_instances,
            table_instances,
            global_instances,
            exports,
          )
          .without_validate()?,
        )
      }
    }
  }
}
