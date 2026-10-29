use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use meta_run::{EnvPolicy, Environment, Run};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitCall {
    git_dir: PathBuf,
    work_tree: Option<PathBuf>,
    words: Vec<OsString>,
    input: Option<String>,
}

impl GitCall {
    const PROGRAM: &'static str = "git";
    const ISOLATED: [(&'static str, &'static str); 4] = [
        ("GIT_CONFIG_GLOBAL", "/dev/null"),
        ("GIT_CONFIG_SYSTEM", "/dev/null"),
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("GIT_TERMINAL_PROMPT", "0"),
    ];
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
        let work_tree = Some(work_tree.to_path_buf());
        Self { git_dir: git_dir.to_path_buf(), work_tree, words, input: None }
    }

    pub fn fed(self, input: String) -> Self {
        Self { input: Some(input), ..self }
    }

    #[cfg(test)]
    pub fn words(&self) -> &[OsString] {
        &self.words
    }

    pub fn common(git_dir: &Path, words: &[&OsStr]) -> Self {
        let words = words.iter().map(|word| word.to_os_string()).collect();
        Self { git_dir: git_dir.to_path_buf(), work_tree: None, words, input: None }
    }

    pub fn run(&self, inherited: Environment) -> Run {
        let run = Run::inheriting(Self::PROGRAM, EnvPolicy::NoGitVars, inherited);
        let run = Self::ISOLATED.iter().fold(run, |run, (name, value)| run.var(name, value));
        let run = run.args(self.arguments()).dir(self.directory());
        self.input.iter().fold(run, |run, input| run.fed(input.clone()))
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
