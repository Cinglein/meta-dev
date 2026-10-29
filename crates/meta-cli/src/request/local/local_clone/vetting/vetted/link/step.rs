#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Lock,
    Unlock,
    Remove,
}
