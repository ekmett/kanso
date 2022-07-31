use std::{cmp::Ordering, num::NonZeroUsize};
use az::wrapping_cast;

pub trait Semigroup: Sized {
  #[must_use]
  fn op(self, b:Self) -> Self;

  fn op_mut(&mut self, b: Self) {
    *self = self.op(b);
  }
  
  #[must_use]
  fn rep(self, n: NonZeroUsize) -> Self {
    let ng = n.get();
    if let Some(n2) = NonZeroUsize::new(ng >> 1) {
      let self2 = self.op(self).rep(n2);
      if ng & 1 == 0 {
        self2
      } else {
        self2.op(self)
      }
    } else {
      self
    }
  }
}

impl <A:Semigroup,B:Semigroup> const Semigroup for (A,B) {
  fn op(self, b:Self) -> Self {
    (self.0.op(b.0), self.1.op(b.1))
  }
  fn op_mut(&mut self, b:Self) {
    self.0.op_mut(b.0);
    self.1.op_mut(b.1);
  }
  fn rep(self, n: NonZeroUsize) -> Self {
    (self.0.rep(n), self.1.rep(n))
  }
}

impl const Semigroup for () {
  fn op(self, _:Self) -> Self { () }
  fn op_mut(&mut self, _: Self) {}
  fn rep(self, _: NonZeroUsize) -> Self { () }
}

impl const Semigroup for ! {
  fn op(self, _:Self) -> Self { self }
  fn op_mut(&mut self, _: Self) {}
  
  fn rep(self, _: NonZeroUsize) -> Self { self }
}

impl <A:Semigroup> const Semigroup for Option<A> {
  fn op(self, other:Self) -> Self {
    match self {
      None => other,
      Some(a) => match other {
        None => self,
        Some(b) => Some(a.op(b))
      }
    }
  }
  fn op_mut(&mut self, other: Self) {
    if let Some(a) = self {
      if let Some(b) = other {
        a.op_mut(b)
      }
    } else { 
      *self = other 
      }
  }
  fn rep(self, n: NonZeroUsize) -> Self {
    // self? is not allowed in a const function
    match self {
      None => None,
      Some(a) => Some(a.rep(n))
    }
  }
}

pub trait Monoid : Semigroup {
  #[must_use]
  fn id() -> Self;
  // this is a canonical Action<u32> for all monoids
  #[must_use]
  fn rep(self,n:usize) -> Self {
    if n == 0 {
      Monoid::id()
    }
    let self2 = Monoid::rep(self.op(self), n >> 1);
    if n & 1 == 0 {
      self2
    } else {
      self2.op(self)
    }
  }
}

impl const Monoid for () {
  fn id() -> Self { () }
}

impl <A:Semigroup> const Monoid for Option<A> {
  fn id() -> Self { None }
  fn rep(self,n:usize) -> Self {
    // Some(Semigroup::rep(self?,NonZeroUsize::new(n)?)) // unfortunately ? is not allowed in const
    match self {
      None => None,
      Some(a) => match NonZeroUsize::new(n) {
        None => None,
        Some(n) => Some(Semigroup::rep(a,n))
      }
    }
  }
}

impl <A:Monoid,B:Monoid> const Monoid for (A,B) {
  fn id() -> Self { (Monoid::id(), Monoid::id()) }
  fn rep(self,n:usize) -> Self {
    (Monoid::rep(self.0,n),Monoid::rep(self.1,n))
  }
}

pub trait Group : Monoid {
  #[must_use]
  fn inv(self) -> Self;

  fn inv_mut(&mut self) {
    *self = self.inv();
  }

  #[must_use]
  fn rep(self,n:isize) -> Self {
    match n.cmp(&0) {
      Ordering::Less => Monoid::rep(self.inv(), -n as usize),
      Ordering::Equal => Monoid::id(),
      Ordering::Greater => Monoid::rep(self, n as usize),
    }
  }
}

impl const Group for () {
  fn inv(self) -> Self { () }
  fn inv_mut(&mut self) {}
}

impl <A:Group,B:Group> const Group for (A,B) {
  fn inv(self) -> Self { (self.0.inv(), self.1.inv()) }
  fn inv_mut(&mut self) { 
    self.0.inv_mut(); 
    self.1.inv_mut();
  }

  fn rep(self,n:isize) -> Self {
    (Group::rep(self.0,n), Group::rep(self.1,n))
  }
}

// a.op(b) = b.op(a)
trait Commutative: Semigroup {} 
impl const Commutative for () {}
impl const Commutative for ! {}
impl <A:Commutative, B:Commutative> Commutative for (A,B) {}

// for NonZeroUsize, etc?
/*
macro_rules! semigroup_impl {
    ($($t:ty)*) => ($(
      impl const Semigroup for $t {
        fn op(self, other: Self) -> Self { self.wrapping_add(other) }
      }
    )*)
}
 */

macro_rules! num_impl {
    ($($t:ty)*) => ($(
      impl const Semigroup for $t {
        fn op(self, other: Self) -> Self { self.wrapping_add(other) }
        fn rep(self, n: NonZeroUsize) -> Self { self.wrapping_mul(wrapping_cast(n.get())) }
      }
      impl const Monoid for $t {
        fn id() -> Self { 0 }
        fn rep(self, n: usize) -> Self { self.wrapping_mul(wrapping_cast(n)) }
      }
      impl const Group for $t {
        fn inv(self) -> Self { self.wrapping_neg() }
        fn rep(self, n: isize) -> Self { self.wrapping_mul(wrapping_cast(n)) }
      }
      impl const Commutative for $t {}

    )*)
}

num_impl! { usize u8 u16 u32 u64 u128 isize i8 i16 i32 i64 i128 } 

// f32 f64

// act(act(x,m),n) = act(x,m.op(n))
pub trait Action<M: Semigroup>: Sized {
  #[must_use]
  fn act(&self,m:M) -> Self;
  fn act_mut(&mut self, m:M) {
    let r = self.act(m);
    *self = r;
  }
}

impl <M: Semigroup, A: Action<M>, B: Action<M>> const Action<M> for (A,B) {
  fn act(&self,m:M) -> Self {
    (self.0.act(m), self.1.act(m))
  }
}

// act(x,id()) = x
pub trait MonoidAction<M: Monoid> : Action<M> {}

impl <M: Monoid, A: MonoidAction<M>, B: MonoidAction<M>> const MonoidAction<M> for (A,B) {}

pub trait GroupAction<M: Group>: MonoidAction<M> {}
impl <M:Group,T:MonoidAction<M>> const GroupAction<M> for T {}

// a.rel(m).op(b.rel(m)) = a.op(b).rel(m)
pub trait RelativeSemigroup<M:Semigroup> : Action<M> + Semigroup {}

impl <M:Semigroup,A:RelativeSemigroup<M>,B:RelativeSemigroup<M>> RelativeSemigroup<M> for (A,B) {}

// id().rel(m) = id()
pub trait RelativeMonoid<M: Semigroup> : RelativeSemigroup<M> + Monoid {}

impl <M:Semigroup,A:RelativeMonoid<M>,B:RelativeMonoid<M>> RelativeMonoid<M> for (A,B) {}

pub trait RelativeGroup<M:Semigroup>: RelativeMonoid<M> + Group {}
impl <M:Group,T:RelativeMonoid<M> + Group> const RelativeGroup<M> for T {}

/*
impl <T> Action<()> for T {
  fn act(self,a:()) -> Self { self }
  fn act_mut(&mut self, a:()) {}
}

impl <T:Semigroup> const Action<NonZeroUsize> for T {
  fn act(self,a:NonZeroUsize) -> Self { self.rep(a) }
}

impl <T:Monoid> const Action<u32> for T {
  fn act(self,a:u32) -> Self { self.rep(a) }
}

impl <T:Group> const Action<i32> for T {
  fn act(self,a:i32) -> Self { self.rep(a) }
}
*/

#[derive(Debug,Copy,Clone,Hash,PartialEq,Eq,PartialOrd,Ord)]
pub struct Semi<A,B>(A,B);

impl <A:Semigroup,B:RelativeSemigroup<A>> const Semigroup for Semi<A,B> {
  fn op(self, b:Self) -> Self {
    Semi(self.0.op(b.0), self.1.op(b.1.act(self.0)))
  }

  fn op_mut(&mut self, b: Self) {
    self.1.op_mut(b.1.act(self.0));
    self.0.op_mut(b.0);
  }

  // todo; optimize rep    
}

impl <A:Monoid,B:RelativeMonoid<A>> const Monoid for Semi<A,B> {
  fn id() -> Self {
    Semi(A::id(),B::id())
  }
}

impl <A:Group,B:RelativeGroup<A>> const Group for Semi<A,B> {
  fn inv(self) -> Self {
    let ai = self.0.inv();
    Self(ai,self.1.inv().act(ai))
  }
  fn inv_mut(&mut self) {
    self.0.inv_mut();
    self.1.inv_mut();
    self.1.act_mut(self.0);
  }
  // todo optimize rep
}


