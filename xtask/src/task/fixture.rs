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
    const WORKING: &str =
        "[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n[maintenance]\n\tauto = false\n";
    const BARE: &str =
        "[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = true\n[maintenance]\n\tauto = false\n";

    pub fn new() -> Fixture {
        let directory = TempDir::new().expect("temp dir");
        let work = directory.path().join("work");
        Self::laid(&work.join(".git"), Self::WORKING);
        Fixture { _directory: directory, work }
    }

    fn laid(git_dir: &Path, config: &str) {
        fs::create_dir_all(git_dir.join("objects")).expect("objects dir");
        fs::create_dir_all(git_dir.join("refs/heads")).expect("refs dir");
        fs::write(git_dir.join("HEAD"), "ref: refs/heads/change\n").expect("write HEAD");
        fs::write(git_dir.join("config"), config).expect("write config");
    }

    pub fn branch(&self) {
        self.committed();
        let tip = fs::read_to_string(self.work.join(".git/refs/heads/change")).expect("read branch tip");
        let remote = self.work.join(".git/refs/remotes/origin");
        fs::create_dir_all(&remote).expect("remote refs dir");
        fs::write(remote.join("main"), tip).expect("write origin main");
        fs::write(remote.join("HEAD"), "ref: refs/remotes/origin/main\n").expect("write origin HEAD");
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
        Self::laid(&bare, Self::BARE);
        self.pointed(&bare);
        other.pointed(&bare);
        bare
    }

    pub fn pointed(&self, origin: &Path) -> &Fixture {
        let remote = format!("[remote \"origin\"]\n\turl = {}\n", origin.display());
        fs::write(self.work.join(".git/config"), [Self::WORKING, &remote].concat()).expect("write config");
        self
    }

    pub fn beside(&self) -> PathBuf {
        self.work.with_file_name("beside")
    }

    pub fn revision(&self, name: &str) -> String {
        let tip = fs::read_to_string(self.work.join(".git/refs/heads/change")).ok().filter(|_| name == "HEAD");
        tip.unwrap_or_else(|| {
            let output = Self::git(&self.work).args(["rev-parse", name]).output().expect("run git");
            String::from_utf8_lossy(&output.stdout).into_owned()
        })
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
