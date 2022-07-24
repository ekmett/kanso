#[allow(unused_imports)] // too lazy for now


extern crate kanso;

extern crate clap;

use kanso::args::*;

pub fn main() -> std::io::Result<()> {
  let args = get_args();

  match &args.command {
    Commands::Run { name } => {
      println!("Running {}",name);
    },
    Commands::Repl => {
      println!("Repling");
    }
    Commands::Auto { shell, path } => {
      autocompletions(*shell, path.as_ref())?;
    }
    Commands::Man { path } => {
      man_page(path.as_ref())?;
    }
  }
  Ok(())
}
