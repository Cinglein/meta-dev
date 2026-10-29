use meta_run::Budget;

use super::Link;
use super::scope::Scope;
use super::worktree_config::WorktreeConfig;
use crate::request::local::local_clone::vetting::asked::PausedRoots;
use crate::request::local::local_clone::vetting::disk::Disk;
use crate::request::local::local_clone::vetting::entry_kind::EntryKind;
use crate::request::local::local_clone::vetting::kept::Kept;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Guard {
    present: Vec<&'static str>,
    roots: Result<(), Kept>,
    config: WorktreeConfig,
}

impl Guard {
    const CONFIG: &'static str = "config.worktree";
    const RULES: [(&'static str, Scope, Kept); 8] = [
        ("locked", Scope::Worktree, Kept::Locked),
        ("reftable", Scope::Worktree, Kept::Unlistable),
        ("objects/info/alternates", Scope::Common, Kept::Borrowed),
        ("modules", Scope::Common, Kept::Submodules),
        ("rebase-merge", Scope::Worktree, Kept::Paused),
        ("rebase-apply", Scope::Worktree, Kept::Paused),
        ("sequencer", Scope::Worktree, Kept::Paused),
        ("MERGE_HEAD", Scope::Worktree, Kept::Paused),
    ];

    pub fn observed(link: &Link, budget: Budget) -> Self {
        let dirs = |scope| match scope {
            Scope::Worktree => vec![link.git_dir()],
            Scope::Common => vec![link.git_dir(), link.common()],
        };
        let present: Vec<&'static str> = Self::RULES
            .iter()
            .filter(|(name, scope, _)| dirs(*scope).iter().any(|dir| Disk::entry(&dir.join(name)) != EntryKind::Missing))
            .map(|(name, ..)| *name)
            .collect();
        let roots = if present.is_empty() {
            link.repo(Scope::Worktree, budget).observe(&PausedRoots)
        } else {
            Ok(())
        };
        let Ok(config) = Disk::text(&link.git_dir().join(Self::CONFIG)).unwrap_or_default().parse();
        Self { present, roots, config }
    }

    #[cfg(test)]
    pub fn new(present: Vec<&'static str>, roots: Result<(), Kept>, config: WorktreeConfig) -> Self {
        Self { present, roots, config }
    }

    pub fn judged(&self) -> Result<(), Kept> {
        let broken = Self::RULES.iter().find(|(name, ..)| self.present.contains(name));
        broken.map_or(Ok(()), |(_, _, kept)| Err(kept.clone()))?;
        self.roots.clone()?;
        self.config.unredirected()
    }
}
