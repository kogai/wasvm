use function::FunctionType;
use inst::Inst;
use std::fmt;
use store::Store;
use trap::Result;
use value::Values;

#[derive(PartialEq, Clone)]
pub struct Frame {
  pub locals: Vec<Values>,
  pub expressions: Vec<Inst>,
  pub return_ptr: usize,
  // TODO: Consider to delete it.
  pub table_addresses: Vec<u32>,
  pub own_type: Option<FunctionType>,
  // continuation: u32,
}

impl Frame {
  pub fn new(
    store: &mut Store,
    return_ptr: usize,
    function_idx: usize,
    locals: &mut Vec<Values>,
  ) -> Result<Self> {
    let function_instance = store.get_function_instance(function_idx)?;
    let own_type = match function_instance.get_function_type() {
      Ok(ref t) => Some(t.to_owned()),
      _ => None,
    };
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
