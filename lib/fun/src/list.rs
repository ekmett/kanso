use ::sync::Lrc;
use ::algebra::*;

#[repr(transparent)]
#[derive(Clone)]
pub struct List<A>(Option<Lrc<(A,List<A>)>>);

impl <A> List<A> {
  #[inline]
  #[must_use]
  pub const fn nil() -> List<A> { List(None) }
  #[inline]
  #[must_use]
  pub fn cons(x:A, xs: List<A>) -> List<A> { List(Some(Lrc::new((x,xs)))) }
}

impl <A:Clone> List<A> {
  //#[must_use]
  //pub fn peek(&self) -> Option<&(A,Self)> { self.0.as_ref() }

  // if you are going to clone the result, and are dropping this reference, you can use this
  #[inline]
  #[must_use]
  pub fn uncons(&self) -> Option<(A,List<A>)> where A:Clone {
    Some(self.0.as_ref()?.as_ref().clone())
  }

  #[inline]
  #[must_use]
  pub fn head(&self) -> Option<A> where A:Clone {
    Some(self.0.as_ref()?.0.clone())
  }

  #[inline]
  #[must_use]
  pub fn tail(&self) -> Option<List<A>> {
    Some(self.0.as_ref()?.1.clone())
  }
}

impl <A> Semigroup for List<A> {
  fn op(self, ys:Self) -> Self {
    match self.0.as_ref() {
      None => ys,
      Some(xxs) => List::cons(xxs.0,xxs.1.op(ys))
    }
  }
}

impl <A> Monoid for List<A> {
  fn id() -> Self { List(None) }
}