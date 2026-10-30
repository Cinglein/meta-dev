#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::task::clippy::diagnostics) enum Standing {
    Placed,
    Unplaced,
    Summary,
}
