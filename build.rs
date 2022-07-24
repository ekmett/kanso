#![allow(dead_code)]

#[macro_use]
extern crate clap;
extern crate clap_complete;
extern crate clap_mangen;

// this is designed to be usable with minimal imports so we can use it both here and at runtime
#[path = "src/args.rs"]
mod args;

fn main() -> std::io::Result<()> {
  args::build_man_page()
}
