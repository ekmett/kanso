use args::*;

#[salsa::query_group(OptsStorage)]
pub trait Opts {
  #[salsa::input]
  fn args(&self) -> Arguments;

  #[salsa::input]
  fn input_file(&self, name: String) -> String;

  #[salsa::input]
  fn length(&self) -> usize;
}
