use std::cell::UnsafeCell;
use std::ops::FnOnce;
use std::boxed::Box;
use std::rc::Rc;
use std::mem;

enum Thunk<T> {
  Delayed(Box<dyn (FnOnce() -> T) + 'static>),
  Forced(T),
}

fn blackhole<T>() -> Box<dyn (FnOnce() -> T) + 'static> {
  Box::new(|| panic!("<infinite loop>"))
}

impl<T> Thunk<T> {
  fn delay<F: FnOnce() -> T + 'static>(f: F) -> Self {
    Thunk::Delayed(Box::new(f))
  }
  fn force(&mut self) -> &T {
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

struct LazyCell<T>(UnsafeCell<Thunk<T>>);

impl<T> LazyCell<T> {
  pub fn delay<F: (FnOnce() -> T) + 'static>(f: F) -> Self {
    LazyCell(UnsafeCell::new(Thunk::delay(f)))
  }

  pub fn force(&self) -> &T {
    unsafe { &mut *self.0.get() }.force()
  }
}

#[repr(transparent)]
pub struct Lazy<T>(Rc<LazyCell<T>>);

impl<T> Lazy<T> {
  pub fn delay<F: (FnOnce() -> T) + 'static>(f: F) -> Self {
    Lazy(Rc::new(LazyCell::delay(f)))
  }
  pub fn force(&self) -> &T { self.0.as_ref().force() }
}

fn main() {
   let mut y = 12;
   let x = Lazy::delay(|| y + 3);
   println!("{}",x.force())


}

