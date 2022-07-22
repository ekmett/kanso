use std::mem::swap;

use id;

// using rem's algorithm
// https://drops.dagstuhl.de/opus/volltexte/2020/11801/pdf/LIPIcs-OPODIS-2019-15.pdf
//
// rem's algorithm has the benefit of becoming more stable over time as existing 
// Ids can only shrink.
pub struct DisjointSets(Vec<Id>);

impl DisjointSets {
  pub fn make_set(&mut self) -> Id {
    let id = Id::of_usize(self.parents.len()).unwrap();
    self.0.push(Cell { parent = id, rank = 0 })
    id
  }
  pub fn len(&self) -> usize { return self.0.len(); }

  pub fn parent(&self, p: Id) -> Id { self.0[p.as_usize()]}
  pub fn parent_mut(&mut self, p: Id) -> &mut Id { &mut self.0[p.as_usize()]}

  pub fn find(&self, mut p: Id) -> Id {
    while p != self.parent(p) {
      p = self.parent(p);
    }
    p
  }

  pub fn find_mut(&mut self, mut current: Id) -> Id {
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
      if u == v || up == vp { break up; }
      if vp < up {
        swap(u,v);
        swap(up,vp);
      }
      if u == up {
        // if compare_and_swap(self.parent_mut(u),u,vp) { return vp; } 
        self.parent_mut(u) = vp;
        break vp;
      }
      v = self.parent(up);
      if up != v {
        // compare_and_swap(self.parent_mut(u), up, v)
        self.parent_mut(u), u.parent = v;
      }
      u = up;
    }
  }

  // match the behavior of a more traditional union_find
  pub fn union_find(&mut self, mut u: Id, mut v: Id) -> Id {
    self.find_mut(self.union(u,v));
  }

  pub fn same(&mut self, mut u: Id, mut v: Id) -> bool {
    loop { 
      let mut up = self.parent(u);
      let mut vp = self.parent(v);
      if u == v or up == vp { break true }
      if vp < up { 
        swap(u,v);
        swap(up,vp);
      }
      if u = up { break false }
      v = self.parent(up);
      if up != v {
        // compare_and_swap(self.parent_mut(u), up, v)
        self.parent_mut(u) = v;
      }
      u = up;
    }
  }
}
