#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Clock {
    Wall,
    Cpu,
}
