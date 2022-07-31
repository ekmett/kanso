
use std::iter::FromIterator;
use hm::Q;
use algebra::*;

// bootstrapped non-empty catenable output-restricted deque
#[derive(Clone)]
pub struct NonEmptyCat<A>(A,Q<NonEmptyCat<A>>);

impl <A> NonEmptyCat<A> {
  #[inline]
  #[must_use]
  fn link(self,other: Self) -> NonEmptyCat<A> where A:Clone {
    NonEmptyCat(self.0,Q::snoc(self.1,other))
  }
  #[inline]
  #[must_use]
  fn singleton(a:A) -> Self {
    NonEmptyCat(a,Q::nil())
  }
  #[inline]
  #[must_use]
  fn cons(a:A, bs:Self) -> Self where A:Clone{
    NonEmptyCat::singleton(a).link(bs)
  }
  #[inline]
  #[must_use]
  fn snoc(self, b:A) -> Self where A:Clone {
    self.link(NonEmptyCat::singleton(b))
  }
  #[inline]
  #[must_use]
  fn peek_head(&self) -> &A { &self.0 }
}

impl <A:Clone> Semigroup for NonEmptyCat<A> {
  fn op(self, other:Self) -> Self {
    self.link(other)
  }
  fn op_mut(&mut self, other: Self) {
    self.1 = Q::snoc(self.1,other);
  }
}

// catenable output-restricted deque
pub struct Cat<A>(Option<NonEmptyCat<A>>);

impl <A> const Default for Cat<A> {
  fn default() -> Self { Cat(None) }
}

#[must_use]
fn linkAll<A:Clone>(q: Q<NonEmptyCat<A>>) -> Cat<A> {
  match q.uncons() {
    None => Cat(None),
    Some((h,t)) => {
      match linkAll(t).0 {
      None => Cat(Some(h)),
      Some(n) => Cat(Some(h.link(n)))
      }
    }
  }
}

impl <A> Cat<A> {
  const fn nil() -> Self { Cat(None) }
  #[inline]
  #[must_use]
  fn cons(a:A, other:Self) -> Self where A:Clone {
    match other.0 {
      None => Cat(Some(NonEmptyCat::singleton(a))),
      Some(b) => Cat(Some(NonEmptyCat::cons(a,b)))
    }
  }
  #[inline]
  #[must_use]
  fn snoc(self, b:A) -> Self where A:Clone {
    match self.0 {
      None => Cat(Some(NonEmptyCat::singleton(b))),
      Some(a) => Cat(Some(NonEmptyCat::snoc(a,b))) 
    }
  }
  #[inline]
  #[must_use]
  fn uncons(&self) -> Option<(A,Cat<A>)> where A:Clone {
    let p = self.0?;
    Some((p.0,linkAll(p.1)))
  }
}

impl <A:Clone> Semigroup for Cat<A> {
  fn op(self, other:Self) -> Self {
    Cat(self.0.op(other.0))
  }
  fn op_mut(&mut self, other: Self) {
    self.0.op_mut(other.0);
  }
}

impl <A:Clone> Monoid for Cat<A> {
  fn id() -> Self { Cat::nil() }
}

impl <A:Clone> FromIterator<A> for Cat<A> {
  fn from_iter<T>(iter: T) -> Self where T: IntoIterator<Item = A> {
    let mut r = Cat::nil();
    for i in iter {
        r = r.snoc(i)
    }
    r
  }
}

#[cfg(test)]
mod tests {
// use super::*;
  #[test]
  fn it_works() {

  }
}