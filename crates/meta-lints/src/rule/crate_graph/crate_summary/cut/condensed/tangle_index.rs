#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TangleIndex(usize);

impl TangleIndex {
    pub fn new(index: usize) -> TangleIndex {
        TangleIndex(index)
    }

    pub fn index(self) -> usize {
        self.0
    }
}
