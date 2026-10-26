use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

#[derive(Debug)]
pub struct Fixture {
    _directory: TempDir,
    work: PathBuf,
}

impl Fixture {
    pub fn new() -> Fixture {
        let directory = TempDir::new().expect("temp dir");
        let work = directory.path().join("work");
        Self::ran(Self::git(directory.path()).args(["init", "--quiet", "--initial-branch", "change"]).arg(&work));
        Fixture { _directory: directory, work }
    }

    pub fn branch(&self) {
        Self::ran(Self::git(&self.work).args(["add", "--all"]));
        Self::ran(Self::git(&self.work).args([
            "-c",
            "user.name=fixture",
            "-c",
            "user.email=fixture@invalid",
            "commit",
            "--quiet",
            "-m",
            "fixture",
        ]));
        Self::ran(Self::git(&self.work).args(["update-ref", "refs/remotes/origin/main", "HEAD"]));
        fs::write(self.work.join(".git/refs/remotes/origin/HEAD"), "ref: refs/remotes/origin/main\n")
            .expect("write origin HEAD");
    }

    pub fn commit(&self, [author, committer]: [&str; 2]) -> &Fixture {
        Self::ran(
            Self::git(&self.work)
                .env("GIT_AUTHOR_NAME", "fixture")
                .env("GIT_COMMITTER_NAME", "fixture")
                .env("GIT_AUTHOR_EMAIL", author)
                .env("GIT_COMMITTER_EMAIL", committer)
                .args(["commit", "--quiet", "--allow-empty", "-m", "fixture"]),
        );
        self
    }

    pub fn write(&self, path: &str, text: &str) -> &Fixture {
        let file = self.path(path);
        fs::create_dir_all(file.parent().expect("parent")).expect("create dirs");
        fs::write(file, text).expect("write file");
        self
    }

    pub fn path(&self, path: &str) -> PathBuf {
        self.work.join(path)
    }

    pub fn root(&self) -> &Path {
        &self.work
    }

    fn git(directory: &Path) -> Command {
        let mut command = Command::new("git");
        command.args(["-c", "core.hooksPath=/dev/null"]).current_dir(directory);
        command
    }

    fn ran(command: &mut Command) {
        command.status().expect("run git").success().then_some(()).expect("git succeeds");
    }
}
