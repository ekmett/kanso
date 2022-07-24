cfg_if::cfg_if! {
  if #[cfg(parallel)] {
     pub use std::sync::Arc as Lrc;
     pub use std::sync::Weak as Weak;
  } else {
     pub use std::rc::Rc as Lrc;
     pub use std::rc::Weak as Lrc;
  }
}
