// raw command-line

use clap::{Args, Command, Parser, FromArgMatches};
use clap_complete::{generate, Shell};
use clap_mangen::Man;
use std::env;
use std::io;
use std::fs;
use std::path::{Path, PathBuf};


#[derive(Parser,Debug,PartialEq,Eq,Hash,Clone)]
#[clap(propagate_version = true)]
pub struct Arguments {
  #[clap(short,long, value_parser, default_value_t = true)]
  pub verbose: bool,

  // #[clap(short,long, value_parser)]
  #[clap(short,long, arg_enum, value_name="SHELL")]
  pub completions: Option<Shell>,

  #[clap(subcommand)]
  pub command: Commands
}

#[derive(Subcommand,Debug,PartialEq,Eq,Hash,Clone)]
pub enum Commands {
  Run { name: String },
  Repl,
  Auto { shell: Shell, path: Option<String> },
  Man { path: Option<String> }
}

pub fn cli<'a>() -> Command<'a> {
  Arguments::augment_args(command!())
}

pub fn get_args() -> Arguments {
  let matches = cli().get_matches();
  
  Arguments::from_arg_matches(&matches)
    .map_err(|err| err.exit())
    .unwrap()
}

// write to path or standard out
pub fn with_path_or_stdout<P,F>(mpath : Option<P>, f: F) -> io::Result<()> where
  P: AsRef<Path>,
  F: FnOnce (&mut dyn io::Write) -> io::Result<()>
{
  match mpath {
    Some(path) => {
      let mut buffer: Vec<u8> = Default::default();
      f(&mut buffer)?;
      fs::write(path.as_ref(), buffer)
    },
    None => f(&mut io::stdout())
  }
}

pub fn write_autocompletions(shell: Shell, buf: &mut dyn io::Write) -> io::Result<()> {
  let mut cfg = cli();
  generate(shell, &mut cfg, "kanso", buf);
  Ok(())
}

pub fn write_man_page(buf: &mut dyn io::Write) -> io::Result<()> {
  Man::new(cli()).render(buf)
}

pub fn autocompletions<P: AsRef<Path>>(shell: Shell, mpath: Option<P>) -> io::Result<()> {
  with_path_or_stdout(mpath, |buf: &mut dyn io::Write| write_autocompletions(shell, buf))
}

pub fn man_page<P: AsRef<Path>>(mpath: Option<P>) -> io::Result<()> {
  with_path_or_stdout(mpath, |buf: &mut dyn io::Write| write_man_page(buf))
}

// for use during builds
pub fn build_man_page() -> io::Result<()> {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").ok_or_else(|| io::ErrorKind::NotFound)?);
    man_page(Some(out_dir.join("kanso.1")))
}
