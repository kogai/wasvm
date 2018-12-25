use decode::{Export, Exports};

pub struct InternalModule {
  exports: Exports,
}

impl InternalModule {
  pub fn new(exports: Exports) -> Self {
    InternalModule { exports }
  }

  pub fn get_export_by_key(&self, invoke: &str) -> Option<(Export, usize)> {
    self.exports.get(invoke).map(|x| x.clone())
  }
}
