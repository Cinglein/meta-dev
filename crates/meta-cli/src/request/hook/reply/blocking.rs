use std::fmt;

use crate::request::output::Listed;

#[derive(Debug)]
pub struct Blocking(Listed);

impl From<Listed> for Blocking {
    fn from(reason: Listed) -> Blocking {
        Blocking(reason)
    }
}

impl fmt::Display for Blocking {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "meta-development: {}", self.0)
    }
}
