use std::num::{NonZeroU32, TryFromIntError };
use std::fmt::{self, Debug, Write};
use std::convert::TryFrom;

// this makes names able to be stored in Option<Id> in the same size by adding a niche

#[derive(Copy,Clone,PartialEq,Eq,PartialOrd,Hash)]
#[repr(transparent)]
pub struct Id(NonZeroU32);

impl Id {
  #[inline]
  pub fn new(i: u32) -> Option<Id> {
    Some(Id(NonZeroU32::new(!i)?))
  }

  #[inline]
  pub unsafe fn new_unchecked(i: u32) -> Self {
    Id(NonZeroU32::new_unchecked(!i))
  }

  #[inline]
  pub fn u32(self) -> u32 { !self.0.get() }

  #[inline]
  pub fn from_u32(i: u32) -> Result<Id,TryFromIntError> {
    Ok(Id(NonZeroU32::try_from(!i)?))
  }
}

// assumes the architecture isn't a microcontroller where usize < u32
impl From<Id> for usize {
  #[inline]
  fn from(id: Id) -> usize { usize::try_from(id.u32()).unwrap() }
}

impl From<Id> for u32 {
  #[inline]
  fn from(id: Id) -> u32 { id.u32() }
}

impl TryFrom<u32> for Id {
  type Error = TryFromIntError;
  #[inline]
  fn try_from(u: u32) -> Result<Id,TryFromIntError> { Id::from_u32(u) }
}

impl TryFrom<usize> for Id {
  type Error = TryFromIntError;
  #[inline]
  fn try_from(u: usize) -> Result<Id,TryFromIntError> { Id::from_u32(u32::try_from(u)?) }
}

impl Default for Id {
  #[inline]
  fn default() -> Self { unsafe { Self::new_unchecked(0) } }
}

impl fmt::Debug for Id {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("Id(")?;
    Debug::fmt(&self.u32(),f)?;
    f.write_char(')')
  }
}
