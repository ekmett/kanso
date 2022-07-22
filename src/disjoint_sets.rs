use std::mem::swap;
use std::convert::TryFrom;

use id::Id;

// Using Rem's algorithm rather than the standard Tarjan tricks.
// https://drops.dagstuhl.de/opus/volltexte/2020/11801/pdf/LIPIcs-OPODIS-2019-15.pdf
//
// rem's algorithm has the benefit of becoming more stable over time as existing 
// Ids can only increase, and because they are complemented, this points down towards
// the bottom of the disjoint_sets structure.

#[derive(Debug, Clone, Default)]
pub struct DisjointSets(Vec<Id>);

impl DisjointSets {
  pub fn with_capacity(capacity: u32) -> Self {
    DisjointSets(Vec::with_capacity(usize::try_from(capacity).unwrap()))
  }

  pub fn new() -> Self {
    DisjointSets(Vec::new())
  }


  pub fn make_set(&mut self) -> Id {
    let id = unsafe { Id::new_unchecked(self.len()) };
    self.0.push(id);
    id
  }

  pub fn len(&self) -> u32 { u32::try_from(self.0.len()).unwrap() }

  pub fn capacity(&self) -> u32 { u32::try_from(self.0.capacity()).unwrap() }

  pub fn parent(&self, p: Id) -> Id { self.0[usize::from(p)] }

  fn parent_mut(&mut self, p: Id) -> &mut Id { &mut self.0[usize::from(p)]}

  // find without self-modification
  pub fn find(&self, mut p: Id) -> Id {
    while p != self.parent(p) {
      p = self.parent(p);
    }
    p
  }

  pub fn find_mut(&mut self, mut p: Id) -> Id {
    while p != self.parent(p) {
      let gp = self.parent(self.parent(p));
      *self.parent_mut(p) = gp;
      p = gp
    }
    p
  }

  // make them equal and returns the first node at which this becomes true
  pub fn union(&mut self, mut u: Id, mut v: Id) -> Id {
    loop {
      let mut up = self.parent(u);
      let mut vp = self.parent(v);
      if u == v || up == vp { 
        break up 
      }
      if vp < up {
        swap(&mut u,&mut v);
        swap(&mut up,&mut vp);
      }
      if u == up {
        // if we're doing this multithreaded then
        // if compare_and_swap(self.parent_mut(u),u,vp) { return vp; } 
        *self.parent_mut(u) = vp;
        break vp;
      }
      v = self.parent(up);
      if up != v {
        // if we're doing this multithreaded then
        // compare_and_swap(self.parent_mut(u), up, v)
        *self.parent_mut(u) = v;
      }
      u = up;
    }
  }

  // match the behavior of a more traditional union_find
  pub fn union_find(&mut self, u: Id, v: Id) -> Id {
    let w = self.union(u,v);
    self.find_mut(w)
  }

  pub fn same(&mut self, mut u: Id, mut v: Id) -> bool {
    loop { 
      let mut up = self.parent(u);
      let mut vp = self.parent(v);
      if u == v || up == vp {
        break true
      }
      if vp < up { 
        swap(&mut u,&mut v);
        swap(&mut up,&mut vp);
      }
      if u == up { 
        break false
      }
      v = self.parent(up);
      if up != v {
        // compare_and_swap(self.parent_mut(u), up, v)
        *self.parent_mut(u) = v;
      }
      u = up;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works() {
    let mut ds = DisjointSets::with_capacity(2);
    assert_eq!(ds.capacity(),2);
    assert_eq!(ds.len(), 0);
    let x = ds.make_set();
    let y = ds.make_set();
    let z = ds.make_set();
    assert!(ds.same(x,x));
    assert!(ds.same(y,y));
    assert!(!ds.same(x,y));
    ds.union(x,y);
    assert!(ds.same(x,y));
    assert!(ds.same(x,x));
    assert!(ds.same(y,y));
    assert!(!ds.same(x,z));
    ds.union(x,z);
    assert!(ds.same(y,z));
    assert_eq!(ds.len(), 3);
    let u = ds.make_set();
    let v = ds.make_set();
    let w = ds.union_find(u,v);
    let vr = ds.find_mut(v);
    assert_eq!(w,vr); // known to be roots
  }
}
