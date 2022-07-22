// extern crate tailcall;
// use tailcall::tailcall;
use std::cmp::{Ordering};
use std::fmt;
use std::rc::Rc;

#[derive(Debug,Ord,PartialOrd)] // TODO: hand roll because these are wrong
pub enum Node<T> {
  Tip(T),
  Bin(T,Tree<T>,Tree<T>)
}

impl <T: PartialEq> PartialEq for Node<T> {
  fn eq(&self, other: &Self) -> bool {
    match self {
      Node::Tip(a) => match other {
         Node::Tip(b) => a == b,
         _ => false
      }
      Node::Bin(a,l,r) => match other {
         Node::Bin(b,u,v) => a == b && l == u && r == v,
         _ => false
      }
    }
  }
}

impl <T: PartialEq

impl <T: Eq> Eq for Node<T> {}

type Tree<T> = Rc<Node<T>>;

#[derive(Debug,PartialEq,Eq)]
pub struct Cell<T> { 
  size: u32,
  tree: Tree<T>,
  rest: Orc<T>
}

type Orc<T> = Option<Rc<Cell<T>>>;

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct List<T>(Orc<T>);

#[inline]
fn tip<T>(a: T) -> Tree<T> { Rc::new(Node::Tip(a)) }
#[inline]
fn bin<T>(a: T, l: Tree<T>, r: Tree<T>) -> Tree<T> { Rc::new(Node::Bin(a,l,r)) }

#[inline]
fn cell<T>(size: u32, tree: Tree<T>, rest: Orc<T>) -> Orc<T> {
  Some(Rc::new(Cell { size, tree, rest }))
}

pub fn cons<T>(head: T, tail: List<T>) -> List<T> {
  List(match tail.0.as_ref() {
    Some(c0) => match c0.rest.as_ref() {
      Some(c1) if c0.size == c1.size => 
        cell(c0.size+c1.size+1,bin(head,c0.tree.clone(),c1.tree.clone()),c1.rest.clone()),
      _ => cell(1,tip(head),tail.0)
    },
    _ => cell(1,tip(head),tail.0)
  })
}

pub fn nil<T>() -> List<T> { List(None) }

fn drop_tree<T>(k0: u32, ts0: u32, t0: Tree<T>, rest0: Orc<T>) -> Orc<T> {
  let mut k: u32 = k0;
  let mut t: Tree<T> = t0;
  let mut ts: u32 = ts0;
  let mut rest: Orc<T> = rest0;
  loop {
    ts >>= 1;
    match &*t {
      Node::Tip(_) => break rest,
      Node::Bin(_,l,r) => {
        let bnd = 1 + ts;
        match k.cmp(&bnd) {
          Ordering::Less => {
            rest = cell(ts,r.clone(),rest);
            if k == 1 {
              break cell(ts,l.clone(),rest);
            } else {
              t = l.clone();
              k -= 1;
              // continue
            }
          }
          Ordering::Equal => break cell(ts,r.clone(),rest),
          Ordering::Greater => {
            t = r.clone();
            k -= ts + 1
            // continue
          }
        }
      }
    }
  }
}

fn drop_spine<T>(x0: Orc<T>, n: u32) -> Orc<T> {
  let mut x = x0.clone();
  let mut k = n;
  if n == 0 {
    return x0
  }
  loop { 
    let c = x.as_ref()?;
    match c.size.cmp(&k) {
      Ordering::Less => {
        k -= c.size;
        x = c.rest.clone()
      },
      Ordering::Equal => {
        break c.rest.clone()
      }
      Ordering::Greater => {
        break drop_tree(k,c.size,c.tree.clone(),c.rest.clone())
      }
    }
  }
}

impl <T> List<T> {
  pub fn new() -> List<T> { List(None) }
  pub fn uncons(&self) -> Option<(&T, List<T>)> {
    self.0.as_ref().map(|c| 
      match &*c.tree {
        Node::Tip(a) => (a, List(c.rest.clone())),
        Node::Bin(a,l,r) => {
          let branch_size = c.size >> 1;
          (a,List(cell(branch_size,l.clone(),cell(branch_size,r.clone(),c.rest.clone()))))
        }
      }
    )
  }
  pub fn length(&self) -> u32 {
    match self.0.as_ref() {
      None => 0,
      Some(c) => c.size + List(c.rest.clone()).length()
    }
  }

  pub fn drop(&self, n: u32) -> List<T> { List(drop_spine(self.0.clone(),n)) }

}

impl <T : Clone> Iterator for List<T> { 
  type Item = T;
  fn next(&mut self) -> Option<T> {
    let (h,t) = self.uncons()?;
    let result = Some(h.clone());
    self.0 = t.0;
    result
  }
  fn size_hint(&self) -> (u32, Option<u32>) {
    let n = usize::from(self.length());
    (n, Some(n))
  }
}


impl <A> fmt::Display for List<A> where
  A : fmt::Display, 
  A : Clone {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut delim = "[";
    for a in self.clone() {
      write!(f,"{}{}",delim,a)?;
      delim = ", ";
    }
    write!(f,"]")
  }
}

impl<A> List<A> {
  pub fn reverse(self) -> List<A> where
    A : Clone {
    let mut acc = nil();
    let mut rest = self;
    loop {
      match rest.uncons() {
        None => { break acc }
        Some((y,ys)) => {
          acc = cons(y.clone(),acc);
          rest = ys
        }
      }
    }
  }
}

macro_rules! list {
  [] => { $crate::list::nil() };
  [ $($x:expr),* ] => {{
    let mut l = $crate::list::nil();
    $(
       l = $crate::list::cons($x,l);
    )*
    l.reverse()
  }}
}
     
impl<A> Default for List<A> {
  fn default() -> List<A> { List(None) } 
}

fn main() {
  println!("{:#?}",cons(1,cons(2,nil())));
  println!("{:#?}",list![1,2]);
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn it_works() {
    let u32_nil : List<u32> = nil();
    assert_eq!(u32_nil,list![]);
    assert_eq!(cons(1,nil()),list![1]);
    assert_eq!(cons(1,cons(2,nil())),list![1,2]);
    assert_ne!(cons(1,cons(2,nil())),list![1]);
    assert_eq!(list![4,5,6],list![1,2,3,4,5,6].drop(3))
  }
}
