use std::fmt;

use crate::request::output::Listed;

#[derive(Debug)]
pub struct Denial(Listed);

impl From<Listed> for Denial {
    fn from(reason: Listed) -> Denial {
        Denial(reason)
    }
}

impl fmt::Display for Denial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "meta-development: {}", self.0)
    }
}
