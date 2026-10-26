mod joined_worktree;

use std::fmt;
use std::path::PathBuf;

use meta_state::State;

use super::local_trail::LocalTrail;
use joined_worktree::JoinedWorktree;

#[derive(Debug)]
pub struct Worktrees {
    joined: Vec<JoinedWorktree>,
    unrecorded: Vec<PathBuf>,
}

impl Worktrees {
    pub fn joined(trail: &LocalTrail, listed: &[PathBuf], state: &State) -> Self {
        let recorded = trail.current();
        let joined = recorded.iter().map(|worktree| JoinedWorktree::new(worktree, state)).collect();
        let unrecorded = listed
            .iter()
            .filter(|path| recorded.iter().all(|worktree| worktree.path() != path.as_path()))
            .cloned()
            .collect();
        Self { joined, unrecorded }
    }
}

impl fmt::Display for Worktrees {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.joined.iter().try_for_each(|joined| writeln!(formatter, "{joined}"))?;
        self.unrecorded.iter().try_for_each(|path| writeln!(formatter, "unrecorded {}", path.display()))
    }
}
