// skew-binary random access lists

use serde::de::{Deserializer, SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize, Serializer};
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt::{self, Debug, Display, Formatter};
use std::iter::TrustedLen;
use std::iter::ExactSizeIterator;
use std::marker::PhantomData;
use std::rc::Rc;

// this is encoding less 'correct' than enum { Tip(T), Bin(T,Tree<T>,Tree<T>) } but
// takes advantage of null compression and allows deriving common instances with the right
// behavior. more correct would be Node<T>(T,Option<(Tree<T>,Tree<T>)), but we'd lose more
// bits

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Node<T>(T, N<T>, N<T>);

type Tree<T> = Rc<Node<T>>;
type N<T> = Option<Tree<T>>;

#[derive(Debug, PartialEq, Eq)]
struct Cell<T> {
  size: usize,
  tree: Tree<T>,
  rest: Orc<T>,
}

type Orc<T> = Option<Rc<Cell<T>>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Skew<T>(Orc<T>);

#[inline]
fn tip<T>(a: T) -> Tree<T> {
  Rc::new(Node(a, None, None))
}

#[inline]
fn bin<T>(a: T, l: Tree<T>, r: Tree<T>) -> Tree<T> {
  Rc::new(Node(a, Some(l), Some(r)))
}

#[inline]
fn cell<T>(size: usize, tree: Tree<T>, rest: Orc<T>) -> Orc<T> {
  Some(Rc::new(Cell { size, tree, rest }))
}

pub fn cons<T>(head: T, tail: Skew<T>) -> Skew<T> {
  Skew(match tail.0.as_ref() {
    Some(c0) => match c0.rest.as_ref() {
      Some(c1) if c0.size == c1.size => cell(
        c0.size + c1.size + 1,
        bin(head, c0.tree.clone(), c1.tree.clone()),
        c1.rest.clone(),
      ),
      _ => cell(1, tip(head), tail.0),
    },
    _ => cell(1, tip(head), tail.0),
  })
}

pub const fn nil<T>() -> Skew<T> {
  Skew(None)
}

fn at_tree<T>(mut k: usize, mut ts: usize, mut t: &Tree<T>) -> Option<&T> {
  // panic!("stahp")
  loop {
    ts >>= 1;
    match t.as_ref() {
      Node(a, ml, mr) => {
        if k == 0 {
          break Some(&a);
        }
        k -= 1;
        if k <= ts {
          match ml {
            Some(lt) => t = lt,
            None => break None,
          }
        } else {
          match mr {
            Some(rt) => {
              t = rt;
              k -= ts;
            }
            None => break None,
          }
        }
      }
    }
  }
}

fn drop_tree<T>(mut k: usize, mut ts: usize, mut t: &Tree<T>, mut rest: Orc<T>) -> Orc<T> {
  loop {
    ts >>= 1;
    match t.as_ref() {
      Node(_, Some(l), Some(r)) => {
        let bnd = 1 + ts;
        match k.cmp(&bnd) {
          Ordering::Less => {
            rest = cell(ts, r.clone(), rest.clone());
            // let lp = l.clone();
            if k == 1 {
              break cell(ts, l.clone(), rest);
            } else {
              t = &l;
              k -= 1;
              // continue down left branch
            }
          }
          Ordering::Equal => break cell(ts, r.clone(), rest),
          Ordering::Greater => {
            t = r;
            k -= bnd;
            // continue down right branch
          }
        }
      }
      _ => break rest, // Tip(_)
    }
  }
}

fn drop_spine<T>(mut x: &Orc<T>, mut k: usize) -> Orc<T> {
  if k == 0 {
    return x.clone();
  }
  loop {
    let c = x.as_ref()?;
    match c.size.cmp(&k) {
      Ordering::Less => {
        k -= c.size;
        x = &c.rest;
      }
      Ordering::Equal => break c.rest.clone(),
      Ordering::Greater => break drop_tree(k, c.size, &c.tree, c.rest.clone()),
    }
  }
}

fn at_spine<T>(mut x: &Orc<T>, mut k: usize) -> Option<&T> {
  loop {
    let c = x.as_ref()?;
    if c.size <= k {
      k -= c.size;
      x = &c.rest;
    } else {
      break at_tree(k, c.size, &c.tree);
    }
  }
}

impl<T> Skew<T> {
  pub const fn new() -> Skew<T> {
    Skew(None)
  }
  pub fn uncons(&self) -> Option<(&T, Skew<T>)> {
    self.0.as_ref().map(|c| match &*c.tree {
      Node(a, Some(l), Some(r)) => {
        let branch_size = c.size >> 1;
        (
          a,
          Skew(cell(
            branch_size,
            l.clone(),
            cell(branch_size, r.clone(), c.rest.clone()),
          )),
        )
      }
      Node(a, _, _) => (a, Skew(c.rest.clone())),
    })
  }

  pub fn drop(&self, n: usize) -> Skew<T> {
    Skew(drop_spine(&self.0, n))
  }

  pub fn at(&self, n: usize) -> Option<&T> {
    at_spine(&self.0, n)
  }

  pub fn reverse(self) -> Skew<T>
  where
    T: Clone,
  {
    let mut acc = nil();
    let mut rest = self;
    loop {
      match rest.uncons() {
        None => break acc,
        Some((y, ys)) => {
          acc = cons(y.clone(), acc);
          rest = ys
        }
      }
    }
  }
}

impl<T: Serialize + Clone> Serialize for Skew<T> {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    let mut seq = serializer.serialize_seq(usize::try_from(self.len()).ok())?;
    for e in self.clone() {
      seq.serialize_element(&e)?;
    }
    seq.end()
  }
}

impl<'de, T: Deserialize<'de> + Clone> Deserialize<'de> for Skew<T> {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    struct SkewVisitor<A>(PhantomData<fn() -> Skew<A>>);

    impl<'d, A: Deserialize<'d> + Clone> Visitor<'d> for SkewVisitor<A> {
      type Value = Skew<A>;

      fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("a list")
      }

      fn visit_seq<B: SeqAccess<'d>>(self, mut seq: B) -> Result<Self::Value, B::Error> {
        let mut xs = nil();
        while let Some(x) = seq.next_element()? {
          xs = cons(x, xs);
        }
        Ok(xs.reverse())
      }
    }

    deserializer.deserialize_seq(SkewVisitor(PhantomData))
  }
}

impl<T: Clone> Iterator for Skew<T> {
  type Item = T;
  fn next(&mut self) -> Option<T> {
    let (h, t) = self.uncons()?;
    let result = Some(h.clone());
    self.0 = t.0;
    result
  }
  fn size_hint(&self) -> (usize, Option<usize>) {
    let n = self.len();
    (n, Some(n))
  }
}

unsafe impl <T:Clone> TrustedLen for Skew<T> { }

impl <T:Clone> ExactSizeIterator for Skew<T> {
  fn len(&self) -> usize {
    match self.0.as_ref() {
      None => 0,
      Some(c) => c.size + Skew(c.rest.clone()).len(),
    }
  }
  fn is_empty(&self) -> bool { self.0.is_none() }
}

impl<A: Display + Clone> Display for Skew<A> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut delim = "skew![";
    for a in self.clone() {
      write!(f, "{}{}", delim, a)?;
      delim = ", ";
    }
    write!(f, "]")
  }
}

#[macro_export]
macro_rules! skew {
  [] => { $crate::skew::nil() };
  [ $($x:expr),* ] => {{
    let mut l = $crate::skew::nil();
    $(
       l = $crate::skew::cons($x,l);
    )*
    // TODO reverse parameter order with macro tricks, then just construct directly
    l.reverse()
  }}
}
pub use skew;

impl<A> Default for Skew<A> {
  fn default() -> Skew<A> {
    Skew(None)
  }
}

pub fn main() {
  println!("{:#?}", cons(1, cons(2, nil())));
  println!("{:#?}", skew![1, 2]);
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn it_works() {
    let u32_nil: Skew<u32> = nil();
    assert_eq!(u32_nil, skew![]);
    assert_eq!(cons(1, nil()), skew![1]);
    assert_eq!(cons(1, cons(2, nil())), skew![1, 2]);
    assert_ne!(cons(1, cons(2, nil())), skew![1]);
    assert_eq!(skew![4, 5, 6], skew![1, 2, 3, 4, 5, 6].drop(3))
  }
}
