use std::cell::UnsafeCell;
use std::ops::FnOnce;
use std::boxed::Box;
use std::rc::Rc;
use std::mem;

pub enum Thunk<'f,T: 'f> {
  Delayed(Box<dyn (FnOnce() -> T) + 'f>),
  Forced(T),
}

pub fn blackhole<'f, T : 'f>() -> Box<dyn (FnOnce() -> T) + 'f> {
  Box::new(|| panic!("<infinite loop>"))
}

impl<'f, T> Thunk<'f, T> {
  fn delay<F: FnOnce() -> T + 'f>(f: F) -> Self {
    Thunk::Delayed(Box::new(f))
  }
  fn force(&'f mut self) -> &T {
     let mut mf = None;
     if let Thunk::Delayed(ref mut fr) = self {
       mf = Some(mem::replace(fr, blackhole()))
     }
     if let Some(f) = mf {
       *self = Thunk::Forced(f())
     }
     if let Thunk::Forced(ref t) = self {
       &t
     } else {
       unreachable!()
     }
  }
}

pub struct LazyCell<'f,T>(UnsafeCell<Thunk<'f,T>>);

impl<'f,T> LazyCell<'f,T> {
  pub fn delay<F: (FnOnce() -> T) + 'f>(f: F) -> Self {
    LazyCell(UnsafeCell::new(Thunk::delay(f)))
  }

  pub fn force(&'f self) -> &'f T {
    unsafe { &mut *self.0.get() }.force()
  }
}

#[repr(transparent)]
pub struct Lazy<'f,T>(Rc<LazyCell<'f,T>>);

impl<'f,T> Lazy<'f,T> {
  pub fn delay<F: (FnOnce() -> T) + 'f>(f: F) -> Self {
    Lazy(Rc::new(LazyCell::delay(f)))
  }
  pub fn force(&'f self) -> &'f T { self.0.as_ref().force() }
}
