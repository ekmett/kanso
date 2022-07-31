use ::sync::Lrc;

#[repr(transparent)]
pub struct List<M,A>(Option<Lrc<(M,A,List<M,A>)>>);

impl <M,A:MonoidAction<M>> List<M,A> {
  pub const fn nil() -> List<A> { List(None) }
  pub fn cons(x:A, xs: List<A>) -> List<A> { List(Some(Rc::new((x,xs)))) }

  // if you are going to clone the result, and are dropping this reference, you can use this
  pub fn take(&self) -> Option<(A,List<A>)> where A:Clone {
    Some(self.0?.unwrap_or_clone())
  }
  pub fn tail(&self) -> Option<&List<A>> {
    Some(self.0?.as_ref()).1
  }

  pub fn head(self) -> Option<A> where A:Clone {
    match self.0?.try_unwrap() {
    Err((m,a,_)) => { 
      Some(a.act(m)) 
    },
    Ok(p) => p.1.act(p.0)
    }
  }

  pub fn take_tail(self) -> Option<&List<A>> {
    match self.0?.try_unwrap() {
    Err(rp) => Some(rp.as_ref().1.clone()),
    Ok(p) => p.1
    }
  }
}
