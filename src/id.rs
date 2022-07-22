use std::num::NonZeroU32;
use std::fmt::{self, Debug, Write};

// this makes names able to be stored in Option<Id> in the same size by adding a niche

#[derive(Copy,Clone,PartialEq,Eq,PartialOrd,Hash)]
#[repr(transparent)]
pub struct Id(NonZeroU32);

impl Id {
  pub const fn into_inner(self) -> NonZeroU32 { self.0 }
}

pub unsafe trait Key : Copy + Eq {
  fn as_usize(self) -> usize;
  fn of_usize(i: usize) -> Option<Self>;
}

unsafe impl Key for Id {
  #[inline]
  fn as_usize(self) -> usize { self.0.get() as usize - 1 }

  #[inline]
  fn of_usize(i: usize) -> Option<Self> {
    if i < u32::max_value() as usize {
      unsafe { Some(Id(NonZeroU32::new_unchecked(i as u32 + 1))) }
    } else {
      None
    }
  }
}

impl From<Id> for usize {
  fn from(id: Id) -> usize { id.0.get() }
}

impl Default for Id {
  #[inline]
  fn default() -> Self {
    Self::from_usize(0).unwrap()
  }
}

impl fmt::Debug for Id {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("Id(")?;
    Debug::fmt(&self.0,f)?;
    f.write_char(')')
  }
}
