use std::marker::PhantomData;

pub trait Lens {
  type S;
  type T = <Self as Lens>::S;
  type A;
  type B = <Self as Lens>::A;
  fn get(&self,s:Self::S)->Self::A;
  fn map<F:FnOnce(Self::A)->Self::B>(&self,s:Self::S,f:F)->Self::T;
  fn set(&self,s:Self::S,b:Self::B)->Self::T { self.map(s,move|_|b) }
}

pub struct Comp<P,Q>(P,Q);

impl <P,Q> Lens for Comp<P,Q> where Q:Lens, P: Lens<S=Q::A,T=Q::B>, {
  type S = Q::S;
  type T = Q::T;
  type A = P::A;
  type B = P::B;
  fn get(&self,s:Self::S)->Self::A { self.0.get(self.1.get(s)) }
  fn map<F:FnOnce(Self::A)->Self::B>(&self,s:Self::S,f:F)->Self::T { self.1.map(s,|u|self.0.map(u,f)) }
  fn set(&self,s:Self::S,b:Self::B)->Self::T { self.1.map(s,|u|self.0.set(u,b)) }
}

// it is time to make a macro

pub struct Fst<A,B,C>(PhantomData<(*mut A,*mut B,*mut C)>);

impl <X,Y,Z> Lens for Fst<X,Y,Z> {
  type S = (X,Z);
  type T = (Y,Z);
  type A = X;
  type B = Y;
  fn get(&self,s:Self::S)->Self::A { s.0 }
  fn map<F:FnOnce(Self::A)->Self::B>(&self,s:Self::S,f:F)->Self::T { (f(s.0),s.1) }
  fn set(&self,s:Self::S,b:Self::B)->Self::T { (b,s.1) }
}

fn fst<A,B,C>() -> impl Lens<S=(A,C),T=(B,C),A=A,B=B> { 
  Fst::<A,B,C>(PhantomData) 
}

fn comp<P,Q>(p:P,q:Q) -> impl Lens<S=Q::S,T=Q::T,A=P::A,B=P::B> where Q:Lens, P:Lens<S=Q::A,T=Q::B> {
  Comp(p,q)
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn it_works() {
    assert_eq!(fst::<i32,i32,i32>().get((1,2)),1);
    assert_eq!(comp(fst::<i32,i32,i32>(),fst()).get(((1,2),3)),1);
  }
}

