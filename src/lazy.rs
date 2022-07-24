use std::cell::UnsafeCell;
use std::ops::{Deref, FnOnce, FnMut, Fn};
use std::boxed::Box;
use std::rc::Rc;
use std::fmt::{self,Debug,Formatter};
use std::mem;
use std::borrow::Borrow;

// almost has the semantics of a scala lazy val
// but requires mutation
pub enum Closure<'f,T> {
  Delayed(Box<dyn (FnOnce() -> T) + 'f>),
  Forced(T),
}

impl <'f,T:Debug> Debug for Closure<'f,T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Closure::Delayed(_) => f.write_str("<closure>"),
      Closure::Forced(t)  => Debug::fmt(&t,f)
    }
  }
}

impl<'f, T: 'f> Closure<'f, T> {

  #[inline]
  pub fn new<F: (FnOnce () -> T) + 'f>(f: F) -> Self {
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
  pub fn map_consume<'g,U,F>(self, f: F) -> Closure<'g,U> where
      'f:'g,
      U:'g,
      T:'g,
      F: (FnOnce (&T) -> U) + 'g,
  {
    Closure::new(move || f(&self.consume()))
  }

  pub fn promote(&mut self) -> Lazy<'f, T> where
      T: 'f + Clone
  {
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

impl <'f,T:'f> From<T> for Closure<'f,T> {
  #[inline]
  fn from(that: T) -> Self { Closure::Forced(that) }
}

impl <'f,T:'f+Default> Default for Closure<'f,T> {
  fn default() -> Self { Closure::new(|| T::default()) }
}

impl <'f,T:'f> IntoIterator for Closure<'f,T> {
  type Item = T;
  type IntoIter = detail::ClosureIterator<'f,T>;
  fn into_iter(self) -> Self::IntoIter {
    detail::ClosureIterator(Some(self))
  }
}



// this is a scala-style 'lazy val'. with all the upsides
// and downsides that would entail
#[derive(Debug)]
pub struct LazyVal<'f, T>(UnsafeCell<Closure<'f, T>>);

impl<'f,T:'f> LazyVal<'f, T> {
  pub fn new<F: (FnOnce() -> T) + 'f>(f: F) -> Self {
    LazyVal(UnsafeCell::new(Closure::new(f)))
  }
  pub fn seq(&self) {
    unsafe { &mut *self.0.get() }.seq()
  }
  pub fn ready(&self) -> bool {
    unsafe { &*self.0.get() }.ready()
  }
  pub fn get(&self) -> &'f T {
    unsafe { &mut *self.0.get() }.get()
  }
  pub fn try_get(&self) -> Option<&'f T> {
    unsafe { &*self.0.get() }.try_get()
  }
  pub fn consume(self) -> T {
    self.0.into_inner().consume()
  }
  pub fn try_consume(self) -> Option<T> {
    self.0.into_inner().try_consume()
  }
  pub fn map_consume<'g,U,F>(self, f: F) -> LazyVal<'g,U> where
     'f: 'g, U: 'g, T: 'g,
     F: (FnOnce (&'f T) -> U) + 'g,
  {
    LazyVal::new(move || f(self.get()))
  }
  pub fn promote(&self) -> Lazy<'f, T> where
     T: 'f + Clone
  {
    unsafe { &mut *self.0.get() }.promote()
  }
}

impl <'f,T:Default+'f> Default for LazyVal<'f,T> {
  fn default() -> Self { LazyVal::new(|| T::default()) }
}

impl <'f,T:'f> From<Closure<'f,T>> for LazyVal<'f,T> {
  fn from(that: Closure<'f,T>) -> Self {
    LazyVal(UnsafeCell::new(that))
  }
}

impl <'f,T:'f> From<T> for LazyVal<'f,T> {
  fn from(that: T) -> Self {
    LazyVal::from(Closure::from(that))
  }
}

impl <'f,T:'f> From<LazyVal<'f,T>> for Closure<'f,T> {
  fn from(that: LazyVal<'f,T>) -> Self { that.0.into_inner() }
}

impl <'f,T:'f> Borrow<T> for LazyVal<'f, T> {
  fn borrow(&self) -> &T { self.get() }
}

impl <'f,T:'f> AsRef<T> for LazyVal<'f, T> {
  fn as_ref(&self) -> &T { self.get() }
}

impl <'f,T:'f> Deref for LazyVal<'f, T> {
  type Target = T;
  fn deref(&self) -> &T { self.get() }
}

impl <'f,T:'f> IntoIterator for LazyVal<'f,T> {
  type Item = T;
  type IntoIter = detail::ClosureIterator<'f,T>;
  fn into_iter(self) -> Self::IntoIter {
    self.0.into_inner().into_iter()
  }
}

// a haskell-style thunk, single threaded
#[derive(Debug)]
#[repr(transparent)]
pub struct Lazy<'f,T:'f>(pub Rc<LazyVal<'f, T>>);

impl <'f,T:'f> Clone for Lazy<'f,T> {
  fn clone(&self) -> Self {
    Lazy(self.0.clone())
  }

  fn clone_from(&mut self, source: &Self) {
    self.0.clone_from(&source.0)
  }
}

impl<'f,T:'f> Lazy<'f,T> {
  pub fn new<F: (FnOnce () -> T) + 'f>(f: F) -> Self {
    Lazy(Rc::new(LazyVal::new(f)))
  }
  pub fn new_strict(value: T) -> Self {
    Lazy(Rc::new(LazyVal::from(value)))
  }
  pub fn seq(&self) { self.0.as_ref().seq() }
  pub fn ready(&self) -> bool { self.0.as_ref().ready() }
  pub fn get(&self) -> &'f T { self.0.as_ref().get() }
  pub fn try_get(&self) -> Option<&'f T> { self.0.as_ref().try_get() }
  pub fn map<'g, U, F: (FnOnce (&T) -> U) + 'g>(&self, f: F) -> Lazy<'g,U> where
      'f: 'g, T: 'g, U: 'g
  {
    let me = self.clone();
    Lazy::new(move || f(me.get()))
  }
  pub fn map2<'g,'h,U,V,F: (FnOnce (&T,&U) -> V) + 'h>(
    this: Lazy<'f, T>,
    that: &Lazy<'g,U>,
    f: F
  ) -> Lazy<'h, V> where
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

impl <'f,T:'f+Default> Default for Lazy<'f,T> {
  fn default() -> Self { Lazy::new(|| T::default()) }
}

impl <'f,T:'f> From<T> for Lazy<'f,T> {
  #[inline]
  fn from(that: T) -> Self { Lazy::new_strict(that) }
}

impl<'f,T:'f> FnOnce<()> for Lazy<'f,T> {
  type Output = &'f T;
  extern "rust-call" fn call_once(self, _args: ()) -> &'f T {
    self.0.as_ref().get()
  }
}

impl<'f,T:'f> FnMut<()> for Lazy<'f,T> {
  extern "rust-call" fn call_mut(&mut self, _args: ()) -> &'f T {
    self.0.as_ref().get()
  }
}

impl<'f,T:'f> Fn<()> for Lazy<'f,T> {
  extern "rust-call" fn call(&self, _args: ()) -> &'f T {
    self.0.as_ref().get()
  }
}

impl <'f, T:'f> Borrow<T> for Lazy<'f, T> {
  fn borrow(&self) -> &T { self.get() }
}

impl <'f, T:'f> AsRef<T> for Lazy<'f, T> {
  fn as_ref(&self) -> &T { self.get() }
}

impl <'f, T:'f> Deref for Lazy<'f, T> {
  type Target = T;
  fn deref(&self) -> &T { self.get() }
}

impl <'f, T:'f+Clone> IntoIterator for Lazy<'f,T> {
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
    ...
  }
}
*/


pub mod detail {
  use super::*;

  pub fn blackhole<'f, T:'f>() -> Box<dyn (FnOnce() -> T) + 'f> {
    Box::new(|| panic!("<infinite loop>"))
  }

  pub fn promoting<'f, T:'f>() -> Box<dyn (FnOnce() -> T) + 'f> {
    Box::new(|| unreachable!() )
  }

  #[derive(Debug)]
  #[repr(transparent)]
  pub struct ClosureIterator<'f,T:'f>(pub Option<Closure<'f,T>>);

  impl <'f,T:'f> Iterator for ClosureIterator<'f,T> {
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

  #[derive(Debug)]
  #[repr(transparent)]
  pub struct LazyIterator<'f, T:'f>(pub Option<Lazy<'f,T>>);

  impl <'f, T:'f> Clone for LazyIterator<'f, T> {
    #[inline]
    fn clone(&self) -> Self { LazyIterator(self.0.clone()) }

    #[inline]
    fn clone_from(&mut self, source: &Self) { self.0.clone_from(&source.0) }
  }
  impl <'f, T:'f+Clone> Iterator for LazyIterator<'f,T> {
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

pub fn main() {
  let mut y = 12;
  println!("{}",y);
  let x = Lazy::new(|| { println!("x forced"); y += 1; y * 10 } );
  let w = x.map(|r| r + 1);
  println!("{}",w());
  println!("{}",w.get());
  println!("{}",*w); // deref makes for nice syntax
  for z in w {
    println!("{}",z);
  }
}
