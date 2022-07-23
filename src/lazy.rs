use std::cell::UnsafeCell;
use std::ops::FnOnce;
use std::boxed::Box;
use std::rc::Rc;
use std::mem;

// almost has the semantics of a scala lazy val
// but requires mutation
enum Closure<'f, T> {
  Delayed(Box<dyn (FnOnce() -> T) + 'f>),
  Forced(T),
}

fn blackhole<'f, T>() -> Box<dyn (FnOnce() -> T) + 'f> {
  Box::new(|| panic!("<infinite loop>"))
}

impl<'f, T> Closure<'f, T> {
  fn new<F: FnOnce() -> T + 'f>(f: F) -> Self {
    Closure::Delayed(Box::new(f))
  }
  fn new_strict(value: T) -> Self {
    Closure::Forced(value)
  }

  fn ready(&self) -> bool {
    match self {
    Closure::Forced(_) => true,
    _ => false
    }
  }

  fn seq(&mut self) {
    let mut mf = None;
    if let Closure::Delayed(ref mut fr) = self {
      mf = Some(mem::replace(fr, blackhole()))
    }
    if let Some(f) = mf {
      *self = Closure::Forced(f())
    }
  }
  fn get(&mut self) -> &T {
    self.seq();
    if let Closure::Forced(ref t) = self {
      &t
    } else {
      unreachable!()
    }
  }
  fn try_get(&self) -> Option<&T> {
    if let Closure::Forced(ref t) = self {
      Some(&t)
    } else {
      None
    }
  }
  pub fn consume(self) -> T {
    match self {
      Closure::Delayed(f) => f(),
      Closure::Forced(t) => t
    }
  }

  pub fn try_consume(self) -> Option<T> {
    match self {
      Closure::Forced(t) => Some(t),
      _ => None
    }
  }
  pub fn map_consume<'g, U, F: (FnOnce (&T) -> U) + 'g>(self, f: F) -> Closure<'g,U> where 'f: 'g, T: 'g, {
    Closure::new(move || f(&self.consume()))
  }
}

impl <'f,T: Default> Default for Closure<'f,T> {
  fn default() -> Self { Closure::new(|| T::default()) }
}


pub struct ClosureIterator<'f,T>(Option<Closure<'f,T>>);

impl <'f,T> IntoIterator for Closure<'f,T> {
  type Item = T;
  type IntoIter = ClosureIterator<'f,T>;
  fn into_iter(self) -> Self::IntoIter {
    ClosureIterator(Some(self))
  }
}

impl <'f,T> Iterator for ClosureIterator<'f,T> {
  type Item = T;
  fn next(&mut self) -> Option<Self::Item> {
    Some(self.0.take()?.consume())
  }
  fn size_hint(&self) -> (usize, Option<usize>) {
    let n = if self.0.is_some() { 1 } else { 0 };
    (n,Some(n))
  }
}

// this is a scala-style 'lazy val'. with all the upsides
// and downsides that would entail
struct LazyVal<'f, T>(UnsafeCell<Closure<'f, T>>);

impl<'f, T> LazyVal<'f, T> {
  pub fn new<F: (FnOnce() -> T) + 'f>(f: F) -> Self {
    LazyVal(UnsafeCell::new(Closure::new(f)))
  }
  pub fn new_strict(value: T) -> Self {
    LazyVal(UnsafeCell::new(Closure::new_strict(value)))
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
}

impl <'f,T: Default> Default for LazyVal<'f,T> {
  fn default() -> Self { LazyVal::new(|| T::default()) }
}

impl <'f,T> IntoIterator for LazyVal<'f,T> {
  type Item = T;
  type IntoIter = ClosureIterator<'f,T>;
  fn into_iter(self) -> Self::IntoIter {
    self.0.into_inner().into_iter()
  }
}

/*
// this eats the LazyVal
impl<'f,T> FnOnce<()> for LazyVal<'f,T> {
  type Output = T;
  extern "rust-call" fn call_once(self, _args: ()) -> T { self.get() }
}
*/

// a haskell-style thunk, single threaded
#[repr(transparent)]
pub struct Lazy<'f, T>(Rc<LazyVal<'f, T>>);

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
    Lazy(Rc::new(LazyVal::new_strict(value)))
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

#[repr(transparent)]
pub struct LazyIterator<'f, T>(Option<Lazy<'f,T>>);

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
}

impl <'f, T: Clone> IntoIterator for Lazy<'f,T> {
  type Item = T;
  type IntoIter = LazyIterator<'f,T>;
  fn into_iter(self) -> Self::IntoIter {
    LazyIterator(Some(self))
  }
}


// can't implement alongside the above, because of the conflict with the builtin definitions for FnOnce<A> for &F

/*
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

