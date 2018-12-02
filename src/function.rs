use code::ValueTypes;
use inst::Inst;
use trap::Result;

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionType {
  parameters: Vec<ValueTypes>,
  returns: Vec<ValueTypes>,
}

impl FunctionType {
  pub fn new(parameters: Vec<ValueTypes>, returns: Vec<ValueTypes>) -> Self {
    FunctionType {
      parameters,
      returns,
    }
  }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionInstance {
  export_name: Option<String>,
  pub function_type: Result<FunctionType>,
  locals: Vec<ValueTypes>,
  type_idex: u32,
  body: Vec<Inst>,
}

impl FunctionInstance {
  pub fn new(
    export_name: Option<String>,
    function_type: Result<FunctionType>,
    locals: Vec<ValueTypes>,
    type_idex: u32,
    body: Vec<Inst>,
  ) -> Self {
    FunctionInstance {
      export_name,
      function_type,
      locals,
      type_idex,
      body,
    }
  }

  pub fn call(&self) -> (Vec<Inst>, Vec<ValueTypes>) {
    (self.body.to_owned(), self.locals.to_owned())
  }

  pub fn find(&self, key: &str) -> bool {
    // FIXME: When using function_index, we might get exported function by O(1).
    match &self.export_name {
      Some(name) => name.as_str() == key,
      _ => false,
    }
  }
}
