#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Extent {
    Undeclared,
    Partial,
    Whole,
}
