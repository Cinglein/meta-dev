#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Lock,
    Unlock,
    Remove,
}

impl Step {
    const LOCK: [&'static str; 4] = ["worktree", "lock", "--reason", "meta gc quarantine"];
    const UNLOCK: [&'static str; 2] = ["worktree", "unlock"];
    const REMOVE: [&'static str; 2] = ["worktree", "remove"];

    pub fn words(self) -> &'static [&'static str] {
        match self {
            Self::Lock => &Self::LOCK,
            Self::Unlock => &Self::UNLOCK,
            Self::Remove => &Self::REMOVE,
        }
    }
}
