// the api is inspired by the rust `hashconsing` library, but
// here we take pains to keep the structure null-pointer optimizable
// for immutable structures: replace Rc<T> with Hc<T> and allocate with
// a Constable and your code should be more or less as before with
// increased sharing.

use std::borrow::Borrow;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{BuildHasher, Hash, Hasher};
use std::ops::Deref;
use std::rc::{Rc, Weak};

// null pointer optimization
#[repr(transparent)]
pub struct Hc<T: ?Sized>(Rc<T>);

impl<T> Hc<T> {
    #[inline]
    pub fn get(&self) -> &T {
        self.0.borrow()
    }
    #[inline]
    pub fn id(&self) -> usize {
        Rc::as_ptr(&self.0).addr()
    } //  as *const () as usize }
    #[inline]
    pub fn downgrade(&self) -> WeakHc<T> {
        WeakHc(Rc::downgrade(&self.0))
    }
    #[inline]
    pub fn strong_count(&self) -> usize {
        Rc::strong_count(&self.0)
    }
}

impl<T> Borrow<T> for Hc<T> {
    #[inline]
    fn borrow(&self) -> &T {
        self.0.borrow()
    }
}

impl<T> AsRef<T> for Hc<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.0.borrow()
    }
}

impl<T> Deref for Hc<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        self.0.deref()
    }
}

impl<T: Display> Display for Hc<T> {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        self.0.fmt(fmt)
    }
}

impl<T: Debug> Debug for Hc<T> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{:?}", self.0)
    }
}

impl<T> Clone for Hc<T> {
    #[inline]
    fn clone(&self) -> Self {
        Hc(self.0.clone())
    }
}

impl<T> PartialEq for Hc<T> {
    #[inline]
    fn eq(&self, rhs: &Self) -> bool {
        Rc::ptr_eq(&self.0, &rhs.0)
    }
}
impl<T> Eq for Hc<T> {}

// weak reference to a hash consed structure
// null pointer optimization
#[repr(transparent)]
pub struct WeakHc<T: ?Sized>(Weak<T>);

impl<T> WeakHc<T> {
    #[inline]
    pub fn upgrade(&self) -> Option<Hc<T>> {
        Some(Hc(self.0.upgrade()?))
    }
    #[inline]
    pub fn id(&self) -> usize {
        Weak::as_ptr(&self.0) as *const () as usize
    }
}

impl<T: Debug> Debug for WeakHc<T> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self.0.upgrade() {
            Some(r) => r.fmt(fmt),
            None => write!(fmt, "<removed>"),
        }
    }
}

impl<T: Display> Display for WeakHc<T> {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self.0.upgrade() {
            Some(r) => r.fmt(fmt),
            None => write!(fmt, "<removed>"),
        }
    }
}

impl<T> Hash for WeakHc<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state)
    }
}

impl<T> PartialEq for WeakHc<T> {
    #[inline]
    fn eq(&self, rhs: &Self) -> bool {
        self.0.as_ptr() == rhs.0.as_ptr()
    }
}
impl<T> Eq for WeakHc<T> {}

pub struct Constable<T: Hash + Eq + Clone, S = RandomState>(HashMap<T, WeakHc<T>, S>);

impl<T: Hash + Eq + Clone> Constable<T, RandomState> {
    #[inline]
    pub fn new() -> Self {
        Constable(HashMap::new())
    }
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Constable(HashMap::with_capacity(capacity))
    }
}

impl<T: Hash + Eq + Clone, S: BuildHasher> Constable<T, S> {
    #[inline]
    pub fn with_hasher(build_hasher: S) -> Self {
        Constable(HashMap::with_hasher(build_hasher))
    }

    #[inline]
    pub fn with_capacity_and_hasher(capacity: usize, build_hasher: S) -> Self {
        Constable(HashMap::with_capacity_and_hasher(capacity, build_hasher))
    }

    /// One of the following must hold:
    ///
    /// - `self.table` is not defined at `key`
    /// - the weak ref in `self.table` at `key` returns `None` when upgraded.
    ///
    /// This is checked in `debug` but not `release`.
    #[inline]
    fn insert(&mut self, key: T, wc: WeakHc<T>) {
        let prev = self.0.insert(key, wc);
        debug_assert!(match prev {
            None => true,
            Some(prev) => prev.upgrade().is_none(),
        })
    }

    /// Attempts to retrieve an *upgradable* value from the map.
    #[inline]
    fn get(&self, key: &T) -> Option<Hc<T>> {
        self.0.get(key)?.upgrade()
    }
}

impl<T: Hash + Display + Eq + Clone, S> Display for Constable<T, S> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "constable:")?;
        for e in self.0.values() {
            write!(fmt, "\n  | {}", e)?;
        }
        Ok(())
    }
}

impl<T: Hash + Debug + Eq + Clone, S> Debug for Constable<T, S> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "constable:")?;
        for e in self.0.values() {
            write!(fmt, "\n  | {:?}", e)?;
        }
        Ok(())
    }
}

pub trait HashConstable<T: Hash>: Sized {
    fn mk_is_new(self, elm: T) -> (Hc<T>, bool);
    fn mk(self, elm: T) -> Hc<T> {
        self.mk_is_new(elm).0
    }
    fn collect(self);
    fn shrink_to_fit(self);
    fn collect_to_fit(self);
    fn reserve(self, additional: usize);
}

impl<'a, T: Hash + Eq + Clone, S: BuildHasher> HashConstable<T> for &'a mut Constable<T, S> {
    fn mk_is_new(self, e: T) -> (Hc<T>, bool) {
        // If the element is known and upgradable return it.
        if let Some(hc) = self.get(&e) {
            debug_assert!(*hc.0 == e);
            return (hc.clone(), false); // add a reference
        }
        let hc = Hc(Rc::new(e.clone()));
        self.insert(e, hc.downgrade());
        (hc, true)
    }

    fn collect(self) {
        self.0.retain(|_, value| value.0.strong_count() > 0)
    }

    fn shrink_to_fit(self) {
        self.0.shrink_to_fit()
    }

    fn collect_to_fit(self) {
        self.collect();
        self.shrink_to_fit();
    }

    fn reserve(self, additional: usize) {
        self.0.reserve(additional)
    }
}
