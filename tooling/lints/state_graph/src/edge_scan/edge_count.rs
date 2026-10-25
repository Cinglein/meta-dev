use derive_more::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display)]
pub struct EdgeCount(usize);

impl EdgeCount {
    pub const fn new(edges: usize) -> EdgeCount {
        EdgeCount(edges)
    }
}
