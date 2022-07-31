use std::convert::TryFrom;
use std::cmp::Ordering;
use std::iter::{FromIterator, FusedIterator, ExactSizeIterator, TrustedLen};
//use algebra::*;

use ::list::List;

// todo reduce cloning with careful lifetimes?
#[derive(Clone)]
enum Rot<A> {
  Idle,
  Reversing(usize,List<A>,List<A>,List<A>,List<A>),
  Appending(usize,List<A>,List<A>),
  Done(List<A>) 
}

impl <A> Rot<A> {
  fn exec(self) -> Rot<A> where A: Clone {
    match self {
      Rot::Reversing(ok,a,fp,c,rp) =>
        match (a.uncons(),c.uncons()) {
          (Some((x,f)),Some((y,r))) => 
            Rot::Reversing(ok+1,f,List::cons(x,fp),r,List::cons(y,rp)),
          (_,          Some((y,_))) => 
            Rot::Appending(ok,fp,List::cons(y,rp)),
          _ => unreachable!()
        },
      Rot::Appending(ok,xfp,rp) =>
        if ok == 0 { 
          Rot::Done(rp) 
        } else {
          let (x,fp) = xfp.uncons().unwrap();
          Rot::Appending(ok-1,fp,List::cons(x,rp))
        },
      _ => self
    }
  }
  fn invalidate(&self) -> Rot<A> where A:Clone{
    match self {
      Rot::Reversing(ok,f,fp,r,rp) => Rot::Reversing(ok-1,f.clone(),fp.clone(),r.clone(),rp.clone()),
      Rot::Appending(ok,fp,rp) =>
        if *ok == 0 {
          Rot::Done(rp.tail().unwrap().clone())
        } else {
          Rot::Appending(ok-1,fp.clone(),rp.clone())
        },
      Rot::Idle => Rot::Idle,
      Rot::Done(x) => Rot::Done(x.clone())
    }        
  }
  fn invalidate_mut(&mut self) where A:Clone {
    match self {
      Rot::Reversing(ok,_,_,_,_) => { *ok -= 1; },
      Rot::Appending(ok,_,rp) =>
        if *ok == 0 {
          *self = Rot::Done(rp.tail().unwrap())
        } else {
          *ok -= 1;
        },
      _ => {}
    }        
  }
}


#[derive(Clone)]
// A Hood-Melville queue
pub struct Q<A> {
  lenf: usize,
  f: List<A>,
  state: Rot<A>,
  lenr: usize,
  r: List<A>
}

impl<A> Q<A> {
  pub fn singleton(a:A) -> Q<A> {
    Q { lenf: 1, f: List::cons(a,List::nil()), state: Rot::Idle, lenr: 0, r: List::nil() }
  }

  pub fn nil() -> Q<A> {
    Q { lenf: 0, f: List::nil(), state: Rot::Idle, lenr: 0, r: List::nil() }
  }

  pub fn is_empty(&self) -> bool {
    self.lenf == 0
  }

  
  pub fn length(&self) -> usize {
    self.lenf + self.lenr
  }
}
impl<A:Clone> Q<A> {
  fn exec2(&self) -> Self {
    match self.state.exec().exec() {
      Rot::Done(newf) => Q { f: newf, state: Rot::Idle, .. *self },
      newstate => Q { state: newstate, .. *self }
    }
  }
  fn exec2_mut(&mut self) {
    match self.state.exec().exec() {
      Rot::Done(newf) => { self.f = newf; self.state = Rot::Idle },
      newstate => self.state = newstate
    }
  }
  fn check(&self) -> Self {
    if self.lenr <= self.lenf {
      self.exec2()
    } else {
      let qp = Q {
        lenf: self.lenf + self.lenr, 
        f: self.f, 
        state: Rot::Reversing(0,self.f,List::nil(),self.r,List::nil()), 
        lenr: 0, 
        r: List::nil()
      };
      qp.exec2()
    }
  }
  fn check_mut(&mut self) {
    if self.lenr > self.lenf {
      *self = Q {
        lenf: self.lenf + self.lenr, 
        f: self.f, 
        state: Rot::Reversing(0,self.f,List::nil(),self.r,List::nil()), 
        lenr: 0, 
        r: List::nil()
      };
    }
    self.exec2_mut()
  }

  pub fn snoc(self,x:A) -> Q<A> {
    Q { lenr: self.lenr+1, r: List::cons(x,self.r), .. self } .check()
  }

  pub fn snoc_mut(&mut self,x:A) {
    self.lenr += 1;
    self.r = List::cons(x,self.r);
    self.check_mut();
  }


  pub fn uncons(self) -> Option<(A, Q<A>)> {
    let (x,fp) = self.f.uncons()?;
    let qp = Q { lenf: self.lenf-1, f: fp, state: self.state.invalidate(), .. self};
    Some((x,qp.check()))
  }
}

impl <A> Default for Q<A> {
  fn default() -> Q<A> { Q::nil() }
}

impl <A:Clone> FromIterator<A> for Q<A> {
  fn from_iter<T>(iter: T) -> Self where T: IntoIterator<Item = A> {
    let mut r = Q::nil();
    for i in iter {
        r = r.snoc(i)
    }
    r
  }
}


#[macro_export]
macro_rules! q {
  [] => { $crate::q::Q::nil() };
  [ $($x:expr),* ] => {{
    let mut l = $crate::q::Q::nil();
    $(
       l = $crate::q::Q::snoc(l,$x);
    )*
    // TODO reverse parameter order with macro tricks, then just construct directly
    l
  }}
}

pub use skew;

impl <A:Clone> Iterator for Q<A> {
  type Item = A;
  fn next(&mut self) -> Option<A> {
    let (x,fp) = self.f.uncons()?;
    self.lenf -= 1;
    self.f = fp;
    self.state.invalidate_mut();
    Some(x)
  }
  fn size_hint(&self) -> (usize, Option<usize>) {
    match usize::try_from(self.lenf+self.lenr) {
    Err(_) => (usize::MAX,None),
    Ok(n) => (n,Some(n))
    }
  }
}

// we keep returning None when exhausted.
impl <A:Clone> FusedIterator for Q<A> {}

unsafe impl <A:Clone> TrustedLen for Q<A> {}

impl <A:Clone> ExactSizeIterator for Q<A> {
    fn len(&self) -> usize {
      self.lenf + self.lenr
    }
  
    fn is_empty(&self) -> bool { self.lenf == 0 }
}

impl <A:Clone> Extend<A> for Q<A> {
    fn extend<T>(&mut self, iter: T) where T: IntoIterator<Item = A> {
      for e in iter {
        self.snoc_mut(e);
      }
    }

    fn extend_one(&mut self, item: A) {
      self.snoc_mut(item);
    }

    fn extend_reserve(&mut self, _: usize) {}
}

impl <A:PartialEq + Clone> PartialEq for Q<A> {
  fn eq(&self, other: &Self) -> bool {
    self.length() == other.length() && 
    Iterator::eq(self.clone(), other.clone())
  }
}
impl <A:Eq + Clone> Eq for Q<A> {}

impl <A:PartialOrd + Clone> PartialOrd for Q<A> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Iterator::partial_cmp(self.clone(),other.clone())
  }
}

impl <A:Ord + Clone> Ord for Q<A> {
  fn cmp(&self, other: &Self) -> Ordering {
    Iterator::cmp(self.clone(),other.clone())
  }
}