#![feature(strict_provenance)]
#![feature(fn_traits)]
#![feature(unboxed_closures)]
#![feature(const_trait_impl)]
#![feature(associated_type_defaults)]
#![feature(never_type)]
#![feature(extend_one)]
#![feature(exact_size_is_empty)]
#![feature(trusted_len)]
// #![feature(fused)]
#![feature(arc_unwrap_or_clone)]


// === Linter configuration
#![allow(dead_code)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(trivial_numeric_casts)]

// extern crate tailcalled;
extern crate az;
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
pub mod hc;
pub mod hm;
pub mod meta;
pub mod skew;
pub mod name;
pub mod sets;
pub mod args;
pub mod cat;
pub mod sync;
pub mod list;
pub mod algebra;
pub mod group_relative;
// pub mod make;
