#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Depth(usize);

impl Depth {
    pub const fn new(levels: usize) -> Depth {
        Depth(levels)
    }

    pub fn deeper(self) -> Depth {
        Depth(self.0 + 1)
    }
}
