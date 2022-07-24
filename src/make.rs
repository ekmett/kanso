use args::*;

#[salsa::query_group(OptsStorage)]
pub trait Opts {
  #[salsa::input]
  fn args(&self) -> Arguments;

  #[salsa::input]
  fn input_string(&self, name: ()) -> String;

  fn length(&self, ()) -> usize;
}

fn length(db: &impl HelloWorld, (): ()) -> usize {
  let input_string = db.input_string(());
  println!("running length");
  input_string.len()
}

#[salsa::database(OptsStorage)]
#[derive(Default)]
struct Db { storage :: salsa::Storage<Self> }

impl salsa::Database for Db {
  fn salsa_runtime(&self) -> &salsa::Runtime<Db> { &self.storage }

}
