use value_type::ValueTypes;

#[derive(PartialEq, Debug, Clone)]
pub enum LabelKind {
  If,
  Else,
  Loop,
  Block,
  Frame,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Label {
  pub(crate) source_instruction: LabelKind,
  // FIXME: To Vec type
  pub(crate) return_type: ValueTypes,
  pub(crate) continuation: u32,
}
