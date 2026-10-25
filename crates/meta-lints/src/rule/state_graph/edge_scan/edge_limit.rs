use std::fmt;

use crate::count::Count;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EdgeLimit(usize);

impl EdgeLimit {
    pub const MAX_PER_VERTEX: EdgeLimit = EdgeLimit(4);

    pub fn exceeded_by(self, count: Count) -> bool {
        count > Count::of_len(self.0)
    }
}

impl fmt::Display for EdgeLimit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
