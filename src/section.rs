use code::ValueTypes;
use element::Element;
use function::{FunctionInstance, FunctionType};
use global::GlobalInstance;
use inst::Inst;
use memory::{Data, Limit, MemoryInstance};
use std::default::Default;
use store::Store;
use table::{TableInstance, TableType};
use trap::Result;

pub struct Section {
  function_types: Option<Result<Vec<FunctionType>>>,
  functions: Option<Result<Vec<u32>>>,
  exports: Option<Result<Vec<(String, usize)>>>, // Pair of (name, index)
  codes: Option<Result<Vec<Result<(Vec<Inst>, Vec<ValueTypes>)>>>>,
  datas: Option<Result<Vec<Data>>>,
  limits: Option<Result<Vec<Limit>>>,
  tables: Option<Result<Vec<TableType>>>,
  globals: Option<Result<Vec<GlobalInstance>>>,
  elements: Option<Result<Vec<Element>>>,
}

impl Default for Section {
  fn default() -> Section {
    Section {
      function_types: None,
      functions: None,
      exports: None,
      codes: None,
      datas: None,
      limits: None,
      tables: None,
      globals: None,
      elements: None,
    }
  }
}

macro_rules! impl_builder {
  ($name: ident, $prop: ident, $ty: ty) => {
    pub fn $name<'a>(&'a mut self, xs: Result<Vec<$ty>>) -> &'a mut Self {
      self.$prop = Some(xs);
      self
    }
  };
}

impl Section {
  impl_builder!(function_types, function_types, FunctionType);
  impl_builder!(functions, functions, u32);
  impl_builder!(exports, exports, (String, usize));
  impl_builder!(codes, codes, Result<(Vec<Inst>, Vec<ValueTypes>)>);
  impl_builder!(datas, datas, Data);
  impl_builder!(limits, limits, Limit);
  impl_builder!(tables, tables, TableType);
  impl_builder!(globals, globals, GlobalInstance);
  impl_builder!(elements, elements, Element);

  fn memory_instances(datas: Vec<Data>, limits: Vec<Limit>) -> Vec<MemoryInstance> {
    datas
      .into_iter()
      .map(|d| MemoryInstance::new(d, &limits))
      .collect::<Vec<_>>()
  }
  fn table_instances(elements: Vec<Element>, tables: Vec<TableType>) -> Vec<TableInstance> {
    elements
      .iter()
      .map(|el| {
        let table_type = tables
          .get(el.table_idx as usize)
          .expect("Table type not found.");
        TableInstance::new(&table_type, el)
      })
      .collect::<Vec<_>>()
  }
  fn function_instances(
    function_types: Vec<FunctionType>,
    functions: Vec<u32>,
    exports: Vec<(String, usize)>,
    codes: Vec<Result<(Vec<Inst>, Vec<ValueTypes>)>>,
  ) -> Vec<FunctionInstance> {
    codes
      .into_iter()
      .enumerate()
      .map(|(idx, code)| {
        let export_name = exports
          .iter()
          .find(|(_, i)| i == &idx)
          .map(|(key, _)| key.to_owned());
        let index_of_type = *functions.get(idx).expect("Index of type can't found.");
        let function_type = function_types
          .get(index_of_type as usize)
          .expect("Function type can't found.")
          .to_owned();

        match code {
          Ok((expression, locals)) => FunctionInstance::new(
            export_name,
            Ok(function_type),
            locals,
            index_of_type,
            expression,
          ),
          Err(err) => FunctionInstance::new(export_name, Err(err), vec![], index_of_type, vec![]),
        }
      })
      .collect::<Vec<_>>()
  }

  pub fn complete(self) -> Store {
    match self {
      Section {
        function_types: Some(Ok(function_types)),
        functions: Some(Ok(functions)),
        exports: Some(Ok(exports)),
        codes: Some(Ok(codes)),
        datas: Some(Ok(datas)),
        limits: Some(Ok(limits)),
        tables: Some(Ok(tables)),
        globals: Some(Ok(globals)),
        elements: Some(Ok(elements)),
      } => {
        let memory_instances = Section::memory_instances(datas, limits);
        let table_instances = Section::table_instances(elements, tables);
        let function_instances =
          Section::function_instances(function_types, functions, exports, codes);

        Store::new(
          function_instances,
          memory_instances,
          table_instances,
          globals,
        )
      }
      _ => unreachable!("Sections did not decode properly."),
    }
  }
}
