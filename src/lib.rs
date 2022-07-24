#![feature(strict_provenance)]
#![feature(fn_traits)]
#![feature(unboxed_closures)]
#![feature(associated_type_defaults)]

// === Linter configuration
#![allow(dead_code)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(trivial_numeric_casts)]

// extern crate tailcalled;
extern crate cfg_if;
extern crate lasso;
extern crate serde;
#[macro_use]
extern crate salsa;
#[macro_use]
extern crate clap;
extern crate clap_mangen;
extern crate clap_complete;

pub mod lazy;
pub mod cons;
pub mod meta;
pub mod skew;
pub mod name;
pub mod sets;
pub mod args;
// pub mod make;
