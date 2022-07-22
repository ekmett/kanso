// extern crate tailcall;

mod list;
mod id;
mod disjoint_sets;

use std::rc::Rc;
use list::*;
use disjoint_sets;

/*

pub type Name = String;
pub type Ix = u32;
pub type Lvl = u32;

#[derive(Debug)]
pub enum Tm {
  Var(Ix),
  App(Term,Term),
  Lam(Name,Term)
}

pub type Term = Rc<Tm>;

trait ToName {
  fn to_name(&self) -> Name;
}

impl ToName for String {
  fn to_name(&self) -> Name { self }
}

impl ToName for &str {
  fn to_name(&self) -> Name { self.to_string() }
}


pub fn lam(n: &str, body: Term) -> Term { lam(n.to_string(), body); }
pub fn lam(n: Name, body: Term) -> Term { Rc::new(Tm::Lam(n,body)) }
pub fn app(fun: Term, arg: Term) -> Term { Rc::new(Tm::App(fun,arg)) }
pub fn var(i: Ix) -> Term { Rc::new(Tm::Var(i)) }

pub type Env = List<Value>;

#[derive(Debug)]
pub enum Val {
  Lam(Env,Name,Term)
}

pub type Value = Rc<Val>;

pub fn apply(fun: Value,arg: Value) -> Value {
  match *fun {
    Val::Lam(e,n,b) => eval(cons(arg,e),b)
  }
}

pub fn lookup(e: Env, i: u32) -> Value {
  panic!("not written")
}

pub fn vlam(e: Env, n: Name, body: Term) -> Value { Rc::new(Val::Lam(e,n,body)) }

pub fn eval(e: Env, t: Term) -> Value {
  match *t {
    Tm::Var(i) => { lookup(e,i) }
    Tm::App(f,x) => { apply(eval(e,f),eval(e,x)) }
    Tm::Lam(n,b) => { vlam(e,n,b) }
  }
}
*/

pub fn main() {
/*
  let i = eval(nil(),lam("x".to_string(),var(0)));
  let k = eval(nil(),lam("x".to_string(),lam("y".to_string(),var(1))));
  let ki = eval(cons(k,cons(i,nil())),app(var(0),var(1)));
  println!("{:#?}",ki);
*/
}
