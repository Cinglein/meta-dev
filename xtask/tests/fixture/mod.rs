use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

#[derive(Debug)]
pub struct Fixture {
    _directory: TempDir,
    work: PathBuf,
}

#[expect(clippy::expect_used, reason = "a fixture that cannot be built is a failed test")]
impl Fixture {
    const SUMMARY: &str = "GITHUB_STEP_SUMMARY";

    pub fn new() -> Fixture {
        let directory = TempDir::new().expect("temp dir");
        let origin = directory.path().join("origin.git");
        let work = directory.path().join("work");
        Self::git(directory.path(), &["init", "--quiet", "--bare", &origin.display().to_string()]);
        Self::git(directory.path(), &["init", "--quiet", "--initial-branch", "main", &work.display().to_string()]);
        Self::git(&work, &["remote", "add", "origin", &origin.display().to_string()]);
        Fixture { _directory: directory, work }
    }

    pub fn branch(&self) {
        Self::git(&self.work, &["add", "--all"]);
        Self::git(
            &self.work,
            &["-c", "user.name=fixture", "-c", "user.email=fixture@invalid", "commit", "--quiet", "-m", "fixture"],
        );
        Self::git(&self.work, &["push", "--quiet", "origin", "main"]);
        Self::git(&self.work, &["remote", "set-head", "origin", "main"]);
        Self::git(&self.work, &["switch", "--quiet", "--create", "change"]);
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

    pub fn xtask(&self, task: &str, summary: Option<&Path>) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_xtask"));
        command.arg(task).current_dir(&self.work).env_remove(Self::SUMMARY);
        summary.into_iter().for_each(|file| {
            command.env(Self::SUMMARY, file);
        });
        command.output().expect("run xtask")
    }

    fn git(directory: &Path, args: &[&str]) {
        let status =
            Command::new("git").args(["-c", "core.hooksPath=/dev/null"]).args(args).current_dir(directory).status();
        status.expect("run git").success().then_some(()).expect("git succeeds");
    }
}
