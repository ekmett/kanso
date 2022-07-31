use ::algebra::*;
use ::list::List as L;

pub struct List<G:Group,A:GroupAction<G>>(G,L<A>);

impl <G:Group,A:GroupAction<G>> List<G,A> {
  pub const fn nil() -> Self { List(Monoid::id(),L::nil()) }
  pub fn cons(x:A, xs: Self) -> Self { 
    List(xs.0,L::cons(x.act(xs.0.inv()),xs.1))
  }
  // if you are going to clone the result, and are dropping this reference, you can use this
  pub fn uncons(self) -> Option<(A,Self)> where A:Clone {
    let p = self.1.take()?;
    Some((p.0.act(self.0), List(self.0,p.1)))
  }
  pub fn head(&self) -> Option<A> where A:Clone {
    Some(self.1.head()?.act(self.0))
  }
  pub fn tail(&self) -> Option<Self> where A:Clone {
    Some(List(self.0,self.1.tail()?))
  }
}

impl <G:Group+Clone,A:GroupAction<G>+Clone> Action<G> for List<G,A> {
  fn act(&self,m:G) -> Self {
    List(m.op(self.0.clone()),self.1.clone())
  }
}

/*
impl <G,A:GroupAction<G>> Semigroup for List<G,A> {
  fn op(self,other: Self) -> Self {
    match self.take() {
      Some((a,as)
    }
  }
}
*/