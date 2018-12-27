use decode::{Export, Exports, Import};

pub struct InternalModule {
  exports: Exports,
  imports: Vec<Import>,
}

impl InternalModule {
  pub fn new(exports: Exports, imports: Vec<Import>) -> Self {
    InternalModule { exports, imports }
  }

  pub fn get_export_by_key(&self, invoke: &str) -> Option<(Export, usize)> {
    self.exports.get(invoke).map(|x| x.clone())
  }
}
