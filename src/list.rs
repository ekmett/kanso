// skew-binary random access lists

use std::cmp::Ordering;
use std::fmt;
use std::rc::Rc;
use std::convert::TryFrom;

// this is encoding less 'correct' than enum { Tip(T), Bin(T,Tree<T>,Tree<T>) } but
// takes advantage of null compression and allows deriving common instances with the right
// behavior. more correct would be Node<T>(T,Option<(Tree<T>,Tree<T>)), but we'd lose more
// bits

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord)]
struct Node<T>(T,N<T>,N<T>);

type Tree<T> = Rc<Node<T>>;
type N<T> = Option<Tree<T>>;

#[derive(Debug,PartialEq,Eq)]
struct Cell<T> {
  size: u32,
  tree: Tree<T>,
  rest: Orc<T>
}

type Orc<T> = Option<Rc<Cell<T>>>;

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct List<T>(Orc<T>);

#[inline]
fn tip<T>(a: T) -> Tree<T> { Rc::new(Node(a,None,None)) }

#[inline]
fn bin<T>(a: T, l: Tree<T>, r: Tree<T>) -> Tree<T> { Rc::new(Node(a,Some(l),Some(r))) }

#[inline]
fn cell<T>(size: u32, tree: Tree<T>, rest: Orc<T>) -> Orc<T> {
  Some(Rc::new(Cell { size, tree, rest }))
}

pub fn cons<T>(head: T, tail: List<T>) -> List<T> {
  List(match tail.0.as_ref() {
    Some(c0) => match c0.rest.as_ref() {
      Some(c1) if c0.size == c1.size =>
        cell(
          c0.size+c1.size+1,
          bin(head,c0.tree.clone(),c1.tree.clone()),
          c1.rest.clone()
        ),
      _ => cell(1,tip(head),tail.0)
    },
    _ => cell(1,tip(head),tail.0)
  })
}

pub const fn nil<T>() -> List<T> { List(None) }

fn at_tree<T>(mut k: u32, mut ts: u32, mut t: &Tree<T>) -> Option<&T> {
  // panic!("stahp")
  loop {
    ts >>= 1;
    match t.as_ref() {
      Node(a,ml,mr) => {
        if k == 0 { break Some(&a) }
        k -= 1;
        if k <= ts {
          match ml {
            Some(lt) => { t = lt },
            None => break None
          }
        } else {
          match mr {
            Some(rt) => { t = rt; k -= ts; },
            None => break None
          }
        }
      }
    }
  }

}

fn drop_tree<T>(mut k: u32, mut ts: u32, mut t: &Tree<T>, mut rest: Orc<T>) -> Orc<T> {
  loop {
    ts >>= 1;
    match t.as_ref() {
      Node(_,Some(l),Some(r)) => {
        let bnd = 1 + ts;
        match k.cmp(&bnd) {
          Ordering::Less => {
            rest = cell(ts,r.clone(),rest.clone());
            // let lp = l.clone();
            if k == 1 {
              break cell(ts,l.clone(),rest);
            } else {
              t = &l;
              k -= 1;
              // continue down left branch
            }
          }
          Ordering::Equal => break cell(ts,r.clone(),rest),
          Ordering::Greater => {
            t = r;
            k -= bnd;
            // continue down right branch
          }
        }
      },
      _ => break rest, // Tip(_)
    }
  }
}

fn drop_spine<T>(mut x: &Orc<T>, mut k: u32) -> Orc<T> {
  if k == 0 {
    return x.clone()
  }
  loop {
    let c = x.as_ref()?;
    match c.size.cmp(&k) {
      Ordering::Less => {
        k -= c.size;
        x = &c.rest;
      },
      Ordering::Equal => {
        break c.rest.clone()
      }
      Ordering::Greater => {
        break drop_tree(k,c.size,&c.tree,c.rest.clone())
      }
    }
  }
}


fn at_spine<T>(mut x: &Orc<T>, mut k: u32) -> Option<&T> {
  loop {
    let c = x.as_ref()?;
    if c.size <= k {
      k -= c.size;
      x = &c.rest;
    } else {
      break at_tree(k,c.size,&c.tree)
    }
  }
}

impl <T> List<T> {
  pub const fn new() -> List<T> { List(None) }
  pub fn uncons(&self) -> Option<(&T, List<T>)> {
    self.0.as_ref().map(|c|
      match &*c.tree {
        Node(a,Some(l),Some(r)) => {
          let branch_size = c.size >> 1;
          (a,List(cell(branch_size,l.clone(),cell(branch_size,r.clone(),c.rest.clone()))))
        }
        Node(a,_,_) => (a, List(c.rest.clone())),
      }
    )
  }
  pub fn length(&self) -> u32 {
    match self.0.as_ref() {
      None => 0,
      Some(c) => c.size + List(c.rest.clone()).length()
    }
  }

  pub fn drop(&self, n: u32) -> List<T> { List(drop_spine(&self.0,n)) }

  pub fn at(&self, n: u32) -> Option<&T> {
    at_spine(&self.0,n)
  }

  pub fn reverse(self) -> List<T> where T : Clone {
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

impl <T : Clone> Iterator for List<T> {
  type Item = T;
  fn next(&mut self) -> Option<T> {
    let (h,t) = self.uncons()?;
    let result = Some(h.clone());
    self.0 = t.0;
    result
  }
  fn size_hint(&self) -> (usize, Option<usize>) {
    let n = usize::try_from(self.length()).unwrap();
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


#[macro_export]
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
pub use list;

impl<A> Default for List<A> {
  fn default() -> List<A> { List(None) }
}

pub fn main() {
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

