#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Worktree,
    Common,
    Remote,
}
