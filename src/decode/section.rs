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

#[derive(Debug)]
pub struct Section {
  function_types: Option<Vec<FunctionType>>,
  functions: Option<Vec<u32>>,
  exports: Option<Vec<(String, usize)>>, // Pair of (name, index)
  codes: Option<Vec<Result<(Vec<Inst>, Vec<ValueTypes>)>>>,
  datas: Option<Vec<Data>>,
  limits: Option<Vec<Limit>>,
  tables: Option<Vec<TableType>>,
  globals: Option<Vec<GlobalInstance>>,
  elements: Option<Vec<Element>>,
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
    pub fn $name<'a>(&'a mut self, xs: Vec<$ty>) -> &'a mut Self {
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

  fn memory_instances(datas: Option<Vec<Data>>, limits: Option<Vec<Limit>>) -> Vec<MemoryInstance> {
    match (datas, limits) {
      (Some(datas), Some(limits)) => datas
        .into_iter()
        .map(|d| MemoryInstance::new(d, &limits))
        .collect::<Vec<_>>(),
      _ => vec![],
    }
  }
  fn table_instances(
    elements: Option<Vec<Element>>,
    tables: Option<Vec<TableType>>,
  ) -> Vec<TableInstance> {
    match (elements, tables) {
      (Some(elements), Some(tables)) => elements
        .iter()
        .map(|el| {
          let table_type = tables
            .get(el.table_idx as usize)
            .expect("Table type not found.");
          TableInstance::new(&table_type, el)
        })
        .collect::<Vec<_>>(),
      _ => vec![],
    }
  }
  fn export_name(idx: usize, exports: &Option<Vec<(String, usize)>>) -> Option<String> {
    (match exports {
      Some(ref exports) => exports,
      None => return None,
    })
    .iter()
    .find(|(_, i)| i == &idx)
    .map(|(key, _)| key.to_owned())
  }
  fn function_type(idx: usize, function_types: &Vec<FunctionType>) -> FunctionType {
    function_types
      .get(idx)
      .expect("Function type can't found.")
      .to_owned()
  }
  fn function_instances(
    function_types: Vec<FunctionType>,
    functions: Vec<u32>,
    exports: Option<Vec<(String, usize)>>,
    codes: Vec<Result<(Vec<Inst>, Vec<ValueTypes>)>>,
  ) -> Vec<FunctionInstance> {
    codes
      .into_iter()
      .enumerate()
      .map(|(idx, code)| {
        let export_name = Section::export_name(idx, &exports);
        let index_of_type = *functions.get(idx).expect("Index of type can't found.");
        let function_type = Section::function_type(index_of_type as usize, &function_types);
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

  fn global_instances(globals: Option<Vec<GlobalInstance>>) -> Vec<GlobalInstance> {
    match globals {
      Some(gs) => gs,
      None => vec![],
    }
  }

  pub fn complete(self) -> Store {
    match self {
      Section {
        function_types: Some(function_types),
        functions: Some(functions),
        exports,
        codes: Some(codes),
        datas,
        limits,
        tables,
        elements,
        globals,
      } => {
        let memory_instances = Section::memory_instances(datas, limits);
        let table_instances = Section::table_instances(elements, tables);
        let function_instances =
          Section::function_instances(function_types, functions, exports, codes);
        let global_instances = Section::global_instances(globals);

        Store::new(
          function_instances,
          memory_instances,
          table_instances,
          global_instances,
        )
      }
      x => unreachable!("Sections did not decode properly.\n{:?}", x),
    }
  }
}
