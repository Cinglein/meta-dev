use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitCall {
    git_dir: PathBuf,
    work_tree: Option<PathBuf>,
    words: Vec<OsString>,
    input: Option<String>,
}

impl GitCall {
    const PROGRAM: &'static str = "git";
    const GIT_PREFIX: &'static str = "GIT_";
    const ISOLATED: [(&'static str, &'static str); 3] =
        [("GIT_CONFIG_GLOBAL", "/dev/null"), ("GIT_CONFIG_SYSTEM", "/dev/null"), ("GIT_CONFIG_NOSYSTEM", "1")];
    const PINNED: [&'static str; 15] = [
        "--no-replace-objects",
        "-c",
        "core.trustctime=true",
        "-c",
        "core.checkStat=default",
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
        Self { git_dir: git_dir.to_path_buf(), work_tree: Some(work_tree.to_path_buf()), words, input: None }
    }

    pub fn fed(self, input: String) -> Self {
        Self { input: Some(input), ..self }
    }

    pub fn input(&self) -> Option<&str> {
        self.input.as_deref()
    }

    pub fn common(git_dir: &Path, words: &[&OsStr]) -> Self {
        let words = words.iter().map(|word| word.to_os_string()).collect();
        Self { git_dir: git_dir.to_path_buf(), work_tree: None, words, input: None }
    }

    pub fn command(&self, inherited: impl IntoIterator<Item = OsString>) -> Command {
        let mut command = Command::new(Self::PROGRAM);
        let cleared = inherited
            .into_iter()
            .filter(|name| name.as_encoded_bytes().starts_with(Self::GIT_PREFIX.as_bytes()))
            .fold(&mut command, |command, name| command.env_remove(name));
        cleared.envs(Self::ISOLATED).args(self.arguments()).current_dir(self.directory());
        command
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
