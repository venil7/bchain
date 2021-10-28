use std::{cmp::max, fmt::Display};

pub trait ShortDisplay: Display {
  fn short_display(&self) -> String {
    let long = format!("{}", self);
    let last = long.len();
    let first = max(0, last as i64 - 12) as usize;
    (&long[first..last]).to_owned()
  }
}

impl<T: Display> ShortDisplay for T {}
