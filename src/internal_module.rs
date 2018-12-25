use decode::{Export, Exports};
use trap::Result;

pub struct InternalModule {
  exports: Exports,
}

impl InternalModule {
  pub fn new(exports: Exports) -> Self {
    InternalModule { exports }
  }

  pub fn get_function_idx(&self, invoke: &str) -> Result<usize> {
    Ok(*self.exports.get(&Export::Function)?.get(invoke)?)
  }
}
