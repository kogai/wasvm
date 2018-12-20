use function::FunctionType;
use inst::Inst;
use std::fmt;
use store::Store;
use trap::Result;
use value::Values;
use value_type::ValueTypes;

#[derive(PartialEq, Clone)]
pub struct Frame {
  pub locals: Vec<Values>,
  pub expressions: Vec<Inst>,
  pub return_ptr: usize,
  // FIXME: May not need to store tables here, use instead of Store.
  pub table_addresses: Vec<u32>,
  pub own_type: FunctionType,
  ptr: usize,
}

impl Frame {
  pub fn new(
    store: &mut Store,
    return_ptr: usize,
    function_idx: usize,
    locals: &mut Vec<Values>,
  ) -> Result<Self> {
    let function_instance = store.get_function_instance(function_idx)?;
    let own_type = function_instance.get_function_type()?;
    let (expressions, local_types) = function_instance.call();
    for local in local_types {
      locals.push(Values::from(local));
    }
    Ok(Frame {
      locals: locals.to_owned(),
      expressions,
      return_ptr,
      table_addresses: vec![0],
      own_type,
      ptr: 0,
    })
  }
}

impl fmt::Debug for Frame {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    // NOTE: Omit to present expressions and types would be worth :thinking: .
    let locals = self
      .locals
      .iter()
      .map(|x| format!("{:?}", x))
      .collect::<Vec<String>>()
      .join(", ");
    write!(
      f,
      "locals:({}) return:{} table{:?}",
      locals, self.return_ptr, self.table_addresses
    )
  }
}
