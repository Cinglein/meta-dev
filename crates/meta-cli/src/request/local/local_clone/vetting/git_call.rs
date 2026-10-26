use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitCall {
    git_dir: PathBuf,
    work_tree: Option<PathBuf>,
    words: Vec<OsString>,
}

impl GitCall {
    const PINNED: [&'static str; 11] = [
        "--no-replace-objects",
        "-c",
        "core.fsmonitor=false",
        "-c",
        "core.untrackedCache=false",
        "-c",
        "core.ignoreStat=false",
        "-c",
        "core.sparseCheckout=false",
        "-c",
        "status.showUntrackedFiles=all",
    ];

    pub fn worktree(git_dir: &Path, work_tree: &Path, words: &[&OsStr]) -> Self {
        let words = words.iter().map(|word| word.to_os_string()).collect();
        Self { git_dir: git_dir.to_path_buf(), work_tree: Some(work_tree.to_path_buf()), words }
    }

    pub fn common(git_dir: &Path, words: &[&OsStr]) -> Self {
        let words = words.iter().map(|word| word.to_os_string()).collect();
        Self { git_dir: git_dir.to_path_buf(), work_tree: None, words }
    }

    pub fn arguments(&self) -> Vec<OsString> {
        let git_dir = Self::flagged("--git-dir=", &self.git_dir);
        let work_tree = self.work_tree.iter().map(|tree| Self::flagged("--work-tree=", tree));
        let pinned = Self::PINNED.iter().map(OsString::from);
        [git_dir].into_iter().chain(work_tree).chain(pinned).chain(self.words.iter().cloned()).collect()
    }

    pub fn directory(&self) -> &Path {
        self.work_tree.as_deref().unwrap_or(&self.git_dir)
    }

    fn flagged(flag: &str, path: &Path) -> OsString {
        let mut flagged = OsString::from(flag);
        flagged.push(path);
        flagged
    }
}
