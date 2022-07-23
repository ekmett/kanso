use lasso::{Key, Rodeo, RodeoReader, RodeoResolver, Spur};
use serde::{Serialize, Deserialize};
use std::hash::Hash;

#[derive(Serialize,Deserialize,Debug,Copy,Clone,PartialEq,Eq,PartialOrd,Ord,Hash)]
#[repr(transparent)]
pub struct Name(Spur);

// TODO: set aside niches for _ as default() and maybe for a-z, use MSBs for counting?
unsafe impl Key for Name {
  unsafe fn into_usize(self) -> usize { self.0.into_usize() }
  fn try_from_usize(int: usize) -> Option<Name> { Some(Name(Spur::try_from_usize(int)?)) }
}

impl Default for Name {
  fn default() -> Self { Self::try_from_usize(0).unwrap() }
}

pub type Names = Rodeo<Name>;
pub type NameReader = RodeoReader<Name>;
pub type NaameResolver = RodeoResolver<Name>;
