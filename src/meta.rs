use std::num::{NonZeroU32, TryFromIntError };
use std::fmt::{self, Debug, Write};
use std::convert::TryFrom;

// this makes names able to be stored in Option<Meta> in the same size by adding a niche

#[derive(Copy,Clone,PartialEq,Eq,PartialOrd,Hash)]
#[repr(transparent)]
pub struct Meta(NonZeroU32);

impl Meta {
  #[inline]
  pub fn new(i: u32) -> Option<Meta> {
    Some(Meta(NonZeroU32::new(!i)?))
  }

  #[inline]
  pub unsafe fn new_unchecked(i: u32) -> Self {
    Meta(NonZeroU32::new_unchecked(!i))
  }

  #[inline]
  pub fn u32(self) -> u32 { !self.0.get() }

  #[inline]
  pub fn from_u32(i: u32) -> Result<Meta,TryFromIntError> {
    Ok(Meta(NonZeroU32::try_from(!i)?))
  }
}

// assumes the architecture isn't a microcontroller where usize < u32
impl From<Meta> for usize {
  #[inline]
  fn from(id: Meta) -> usize { usize::try_from(id.u32()).unwrap() }
}

impl From<Meta> for u32 {
  #[inline]
  fn from(id: Meta) -> u32 { id.u32() }
}

impl TryFrom<u32> for Meta {
  type Error = TryFromIntError;
  #[inline]
  fn try_from(u: u32) -> Result<Meta,TryFromIntError> { Meta::from_u32(u) }
}

impl TryFrom<usize> for Meta {
  type Error = TryFromIntError;
  #[inline]
  fn try_from(u: usize) -> Result<Meta,TryFromIntError> { Meta::from_u32(u32::try_from(u)?) }
}

impl Default for Meta {
  #[inline]
  fn default() -> Self { unsafe { Self::new_unchecked(0) } }
}

impl fmt::Debug for Meta {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("Meta(")?;
    Debug::fmt(&self.u32(),f)?;
    f.write_char(')')
  }
}
