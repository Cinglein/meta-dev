use std::path::{Path, PathBuf};
use std::process::Command;

use meta_door::{Change, Door};
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
        Self::ran(Self::git(directory.path()).args(["init", "--quiet", "--template=", "-b", "change"]).arg(&work));
        Fixture { _directory: directory, work }
    }

    pub fn branch(&self) {
        self.committed().updated("refs/remotes/origin/main");
        Self::ran(Self::git(&self.work).args(["symbolic-ref", "refs/remotes/origin/HEAD", "refs/remotes/origin/main"]));
    }

    pub fn updated(&self, reference: &str) -> &Fixture {
        Self::ran(Self::git(&self.work).args(["update-ref", reference, "HEAD"]));
        self
    }

    pub fn committed(&self) -> &Fixture {
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
        self
    }

    pub fn shared_origin(&self, other: &Fixture) -> PathBuf {
        let bare = self.work.with_file_name("origin.git");
        Self::ran(Self::git(&self.work).args(["init", "--quiet", "--bare"]).arg(&bare));
        self.pointed(&bare);
        other.pointed(&bare);
        bare
    }

    pub fn pointed(&self, origin: &Path) -> &Fixture {
        Self::ran(Self::git(&self.work).args(["config", "remote.origin.url"]).arg(origin));
        self
    }

    pub fn beside(&self) -> PathBuf {
        self.work.with_file_name("beside")
    }

    pub fn revision(&self, name: &str) -> String {
        let output = Self::git(&self.work).args(["rev-parse", name]).output().expect("run git");
        String::from_utf8_lossy(&output.stdout).into_owned()
    }

    pub fn commit(&self, emails: [&str; 2]) -> &Fixture {
        self.run_as(emails, &["commit", "--quiet", "--allow-empty", "-m", "fixture"]);
        self
    }

    pub fn criss_cross(&self, main: [&str; 2], side: [&str; 2]) -> &Fixture {
        let first = self.commit(side).run_as(main, &["commit-tree", "HEAD^{tree}", "-p", "HEAD^", "-m", "main"]);
        self.run_as(side, &["merge", "--quiet", "--no-edit", &first]);
        let second = self.run_as(main, &["commit-tree", "HEAD^{tree}", "-p", "HEAD^1", "-p", &first, "-m", "main"]);
        self.run_as(main, &["update-ref", "refs/remotes/origin/main", &second]);
        self
    }

    fn run_as(&self, [author, committer]: [&str; 2], args: &[&str]) -> String {
        let output = Self::git(&self.work)
            .env("GIT_AUTHOR_NAME", "fixture")
            .env("GIT_COMMITTER_NAME", "fixture")
            .env("GIT_AUTHOR_EMAIL", author)
            .env("GIT_COMMITTER_EMAIL", committer)
            .args(args)
            .output()
            .expect("run git");
        output.status.success().then_some(()).expect("git succeeds");
        String::from_utf8_lossy(&output.stdout).trim().to_owned()
    }

    pub fn write(&self, path: &str, text: &str) -> &Fixture {
        let file = self.path(path);
        Door::changed(file.parent().expect("parent"), Change::DirAll).expect("create dirs");
        Door::write(file, text).expect("write file");
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
