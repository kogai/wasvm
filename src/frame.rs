use function::FunctionType;
use inst::Inst;
use std::fmt;
use store::Store;
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
  pub fn new(store: &mut Store, function_idx: usize, arguments: &Vec<Values>) -> Self {
    unimplemented!();
  }
}

// fn expand_frame(&mut self, function_idx: usize, arguments: Vec<Values>) -> Result<()> {
//     let function_instance = self.store.call(function_idx)?;
//     let own_type = match function_instance.get_function_type() {
//         Ok(ref t) => Some(t.to_owned()),
//         _ => None,
//     };
//     let (expressions, local_types) = function_instance.call();
//     let mut locals = arguments;
//     for local in local_types {
//         let v = match local {
//             ValueTypes::I32 => Values::I32(0),
//             ValueTypes::I64 => Values::I64(0),
//             ValueTypes::F32 => Values::F32(0.0),
//             ValueTypes::F64 => Values::F64(0.0),
//             _ => unreachable!(),
//         };
//         locals.push(v);
//     }
//     let frame = StackEntry::new_fram(Frame {
//         locals,
//         expressions,
//         return_ptr: self.stack.stack_ptr,
//         function_idx,
//         table_addresses: vec![0],
//         own_type,
//     });
//     self.stack.push(frame)?;
//     Ok(())
// }

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
