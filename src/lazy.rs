use std::cell::UnsafeCell;
use std::ops::{Deref, FnOnce, FnMut, Fn};
use std::boxed::Box;
use std::rc::Rc;
use std::mem;
use std::borrow::Borrow;

// almost has the semantics of a scala lazy val
// but requires mutation
pub enum Closure<'f, T> {
  Delayed(Box<dyn (FnOnce() -> T) + 'f>),
  Forced(T),
}

impl<'f, T> Closure<'f, T> {

  #[inline]
  pub fn new<F: FnOnce() -> T + 'f>(f: F) -> Self {
    Closure::Delayed(Box::new(f))
  }

  #[inline]
  pub const fn ready(&self) -> bool {
    match self {
    Closure::Forced(_) => true,
    _ => false
    }
  }

  #[inline]
  pub fn seq(&mut self) {
    let mut mf = None;
    if let Closure::Delayed(ref mut fr) = self {
      mf = Some(mem::replace(fr, detail::blackhole()))
    }
    if let Some(f) = mf {
      *self = Closure::Forced(f())
    }
  }
  #[inline]
  pub fn get(&mut self) -> &T {
    self.seq();
    if let Closure::Forced(ref t) = self {
      &t
    } else {
      unreachable!()
    }
  }
  #[inline]
  pub const fn try_get(&self) -> Option<&T> {
    if let Closure::Forced(ref t) = self {
      Some(&t)
    } else {
      None
    }
  }
  #[inline]
  pub fn consume(self) -> T {
    match self {
      Closure::Delayed(f) => f(),
      Closure::Forced(t) => t
    }
  }

  #[inline]
  pub fn try_consume(self) -> Option<T> {
    match self {
      Closure::Forced(t) => Some(t),
      _ => None
    }
  }
  #[inline]
  pub fn map_consume<'g, U, F: (FnOnce (&T) -> U) + 'g>(self, f: F) -> Closure<'g,U> where 'f: 'g, T: 'g, {
    Closure::new(move || f(&self.consume()))
  }

  pub fn promote(&mut self) -> Lazy<'f, T> where T: Clone + 'f {
    if let Closure::Forced(value) = self {
      Lazy::from(value.clone())
    } else {
      let placeholder = Closure::new(||unreachable!());
      let old_guts = mem::replace(self, placeholder);
      let result = Lazy(Rc::new(LazyVal(UnsafeCell::new(old_guts))));
      let clone = result.clone();
      let new_guts = Closure::new(move || clone.get().clone());
      let _ = mem::replace(self, new_guts);
      result
    }
  }
}

impl <'f,T> From<T> for Closure<'f,T> {
  #[inline]
  fn from(that: T) -> Self { Closure::Forced(that) }
}

impl <'f,T: Default> Default for Closure<'f,T> {
  fn default() -> Self { Closure::new(|| T::default()) }
}

impl <'f,T> IntoIterator for Closure<'f,T> {
  type Item = T;
  type IntoIter = detail::ClosureIterator<'f,T>;
  fn into_iter(self) -> Self::IntoIter {
    detail::ClosureIterator(Some(self))
  }
}



// this is a scala-style 'lazy val'. with all the upsides
// and downsides that would entail
pub struct LazyVal<'f, T>(UnsafeCell<Closure<'f, T>>);

impl<'f, T> LazyVal<'f, T> {
  pub fn new<F: (FnOnce() -> T) + 'f>(f: F) -> Self {
    LazyVal(UnsafeCell::new(Closure::new(f)))
  }
  pub fn seq(&self) {
    unsafe { &mut *self.0.get() }.seq()
  }
  pub fn ready(&self) -> bool {
    unsafe { &*self.0.get() }.ready()
  }
  pub fn get(&self) -> &T {
    unsafe { &mut *self.0.get() }.get()
  }
  pub fn try_get(&self) -> Option<&T> {
    unsafe { &*self.0.get() }.try_get()
  }
  pub fn consume(self) -> T {
    self.0.into_inner().consume()
  }
  pub fn try_consume(self) -> Option<T> {
    self.0.into_inner().try_consume()
  }
  pub fn map_consume<'g, U, F: (FnOnce (&T) -> U) + 'g>(self, f: F) -> LazyVal<'g,U> where 'f: 'g, T: 'g, {
    LazyVal::new(move || f(self.get()))
  }
  pub fn promote(&self) -> Lazy<'f, T> where T: Clone + 'f {
    unsafe { &mut *self.0.get() }.promote()
  }
}

impl <'f,T: Default> Default for LazyVal<'f,T> {
  fn default() -> Self { LazyVal::new(|| T::default()) }
}

impl <'f,T> From<Closure<'f,T>> for LazyVal<'f,T> {
  fn from(that: Closure<'f,T>) -> Self {
    LazyVal(UnsafeCell::new(that))
  }
}

impl <'f,T> From<T> for LazyVal<'f,T> {
  fn from(that: T) -> Self {
    LazyVal::from(Closure::from(that))
  }
}

impl <'f,T> From<LazyVal<'f,T>> for Closure<'f,T> {
  fn from(that: LazyVal<'f,T>) -> Self { that.0.into_inner() }
}

impl <'f, T> Borrow<T> for LazyVal<'f, T> {
  fn borrow(&self) -> &T { self.get() }
}

impl <'f, T> AsRef<T> for LazyVal<'f, T> {
  fn as_ref(&self) -> &T { self.get() }
}

impl <'f, T> Deref for LazyVal<'f, T> {
  type Target = T;
  fn deref(&self) -> &T { self.get() }
}

impl <'f,T> IntoIterator for LazyVal<'f,T> {
  type Item = T;
  type IntoIter = detail::ClosureIterator<'f,T>;
  fn into_iter(self) -> Self::IntoIter {
    self.0.into_inner().into_iter()
  }
}

// a haskell-style thunk, single threaded
#[repr(transparent)]
pub struct Lazy<'f, T>(pub Rc<LazyVal<'f, T>>);

impl <'f,T> Clone for Lazy<'f,T> {
  fn clone(&self) -> Self {
    Lazy(self.0.clone())
  }

  fn clone_from(&mut self, source: &Self) {
    self.0.clone_from(&source.0)
  }
}

impl<'f,T> Lazy<'f, T> {
  pub fn new<F: (FnOnce() -> T) + 'f>(f: F) -> Self {
    Lazy(Rc::new(LazyVal::new(f)))
  }
  pub fn new_strict(value: T) -> Self {
    Lazy(Rc::new(LazyVal::from(value)))
  }
  pub fn seq(&self) { self.0.as_ref().seq() }
  pub fn ready(&self) -> bool { self.0.as_ref().ready() }
  pub fn get(&self) -> &T { self.0.as_ref().get() }
  pub fn try_get(&self) -> Option<&T> { self.0.as_ref().try_get() }
  pub fn map<'g, U, F: (FnOnce (&T) -> U) + 'g>(&self, f: F) -> Lazy<'g,U> where
      'f: 'g,
      T: 'g,
  {
    let me = self.clone();
    Lazy::new(move || f(me.get()))
  }
  pub fn map2<'g,'h, U, V, F: (FnOnce (&T,&U) -> V) + 'h>(this: Lazy<'f, T>, that: &Lazy<'g,U>, f: F) -> Lazy<'h, V> where
    'f: 'h, 'g: 'h, T: 'h, U: 'h
  {
    let a = this.clone();
    let b = that.clone();
    Lazy::new(move || f(a.get(), b.get()))
  }

  // consumes this lazy value in an effort to try to avoid cloning the contents
  pub fn consume(self) -> T where T: Clone {
    match Rc::try_unwrap(self.0) {
      Result::Ok(lval) => lval.consume(),
      Result::Err(this) => this.get().clone() // other references to this thunk exist
    }
  }
  pub fn try_consume(self) -> Option<T> where T: Clone {
    match Rc::try_unwrap(self.0) {
      Result::Ok(lval) => lval.try_consume(),
      Result::Err(this) => Some(this.try_get()?.clone())
    }
  }
}

impl <'f,T: Default> Default for Lazy<'f,T> {
  fn default() -> Self { Lazy::new(|| T::default()) }
}

impl <'f,T> From<T> for Lazy<'f,T> {
  #[inline]
  fn from(that: T) -> Self { Lazy::new_strict(that) }
}

impl<'f,T:Clone> FnOnce<()> for Lazy<'f,T> {
  type Output = T;
  extern "rust-call" fn call_once(self, _args: ()) -> T { self.consume() }
}

impl<'f,T:Clone> FnMut<()> for Lazy<'f,T> {
  extern "rust-call" fn call_mut(&mut self, _args: ()) -> T { self.get().clone() }
}

impl<'f,T:Clone> Fn<()> for Lazy<'f,T> {
  extern "rust-call" fn call(&self, _args: ()) -> T { self.get().clone() }
}

/* 

These definitions are more correct and avoid Clone, but are harder to call and conflict
with a generic pattern supplied in std for impls of FnOnce of &A, etc. given FnOnce for A.

impl<'a,'f,T:'a> FnOnce<()> for &'a Lazy<'f,T> {
  type Output = &'a T;
  extern "rust-call" fn call_once(self, _args: ()) -> Self::Output { self.get() }
}

impl<'a,'f,T> FnMut<()> for &'a Lazy<'f,T> {
  extern "rust-call" fn call_mut(&mut self, _args: ()) -> Self::Output { self.get() }
}

impl<'a,'f,T> Fn<()> for &'a Lazy<'f,T> {
  extern "rust-call" fn call(&self, _args: ()) -> Self::Output {
    self.get()
  }
}
*/


impl <'f, T> Borrow<T> for Lazy<'f, T> {
  fn borrow(&self) -> &T { self.get() }
}

impl <'f, T> AsRef<T> for Lazy<'f, T> {
  fn as_ref(&self) -> &T { self.get() }
}

impl <'f, T> Deref for Lazy<'f, T> {
  type Target = T;
  fn deref(&self) -> &T { self.get() }
}

impl <'f, T: Clone> IntoIterator for Lazy<'f,T> {
  type Item = T;
  type IntoIter = detail::LazyIterator<'f,T>;
  fn into_iter(self) -> Self::IntoIter {
    detail::LazyIterator(Some(self))
  }
}

/*
impl <'f, T : Clone> Future for Lazy<'f,T> {
  type Output = T;
  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let this = self.into_inner()
    if Some(t) = self.try_get()

  }
}
*/

// can't implement alongside the above, because of the conflict with the builtin definitions for FnOnce<A> for &F

pub fn main() {
   let mut y = 12;
   println!("{}",y);
   let x = Lazy::new(|| { println!("x forced"); y += 1; y * 10 } );
   let w = x.map(|r| r + 1);
   println!("{}",w());
   println!("{}",w.get());
   for z in w {
     println!("{}",z);
   }
}

pub mod detail {
  use super::*;

  pub fn blackhole<'f, T>() -> Box<dyn (FnOnce() -> T) + 'f> {
    Box::new(|| panic!("<infinite loop>"))
  }

  pub fn promoting<'f, T>() -> Box<dyn (FnOnce() -> T) + 'f> {
    Box::new(|| unreachable!() )
  }

  #[repr(transparent)]
  pub struct ClosureIterator<'f,T>(pub Option<Closure<'f,T>>);

  impl <'f,T> Iterator for ClosureIterator<'f,T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
      Some(self.0.take()?.consume())
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
      let n = if self.0.is_some() { 1 } else { 0 };
      (n,Some(n))
    }
    fn last(self) -> Option<Self::Item> {
      Some(self.0?.consume())
    }
    fn count(self) -> usize {
      if self.0.is_some() { 1 } else { 0 }
    }
  }

  #[repr(transparent)]
  pub struct LazyIterator<'f, T>(pub Option<Lazy<'f,T>>);

  impl <'f, T> Clone for LazyIterator<'f, T> {
    #[inline]
    fn clone(&self) -> Self { LazyIterator(self.0.clone()) }

    #[inline]
    fn clone_from(&mut self, source: &Self) { self.0.clone_from(&source.0) }
  }
  impl <'f, T: Clone> Iterator for LazyIterator<'f,T> {
    type Item = T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
      Some(self.0.take()?.consume())
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
      let n = if self.0.is_some() { 1 } else { 0 };
      (n,Some(n))
    }
    fn last(self) -> Option<Self::Item> {
      Some(self.0?.consume())
    }
    fn count(self) -> usize {
      if self.0.is_some() { 1 } else { 0 }
    }
  }
}
