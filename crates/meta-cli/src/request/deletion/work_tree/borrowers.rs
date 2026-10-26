use std::path::{Path, PathBuf};

use meta_schema::RemovedKind;

use crate::request::deletion::kept::Kept;

#[derive(Debug)]
pub struct Borrowers;

impl Borrowers {
    const ALTERNATES: [&'static str; 2] = [".git/objects/info/alternates", "objects/info/alternates"];

    pub fn none_of(root: &Path, kind: RemovedKind) -> Result<(), Kept> {
        match kind {
            RemovedKind::Clone => root
                .parent()
                .and_then(|parent| parent.read_dir().ok())
                .into_iter()
                .flatten()
                .flatten()
                .flat_map(|sibling| Self::alternates(&sibling.path()))
                .filter_map(|alternates| std::fs::read_to_string(alternates).ok())
                .all(|listed| Self::unrelated(&listed, root))
                .then_some(())
                .ok_or(Kept::Borrowed),
            RemovedKind::Worktree | RemovedKind::TargetDir => Ok(()),
        }
    }

    fn alternates(sibling: &Path) -> Vec<PathBuf> {
        Self::ALTERNATES.iter().map(|alternates| sibling.join(alternates)).collect()
    }

    fn unrelated(listed: &str, root: &Path) -> bool {
        !listed.lines().any(|line| Path::new(line).canonicalize().is_ok_and(|lent| lent.starts_with(root)))
    }
}
