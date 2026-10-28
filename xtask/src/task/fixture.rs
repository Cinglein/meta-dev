use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, LazyLock, Mutex, OnceLock};

use tempfile::TempDir;

use crate::task::snapshot::Snapshot;

static BUILT: LazyLock<Mutex<BuiltRepositories>> = LazyLock::new(Mutex::default);

type BuiltRepositories = HashMap<Snapshot, Arc<OnceLock<Snapshot>>>;

#[derive(Debug)]
pub struct Fixture {
    _directory: TempDir,
    work: PathBuf,
}

impl Fixture {
    const TIP: &str = ".git/refs/heads/change";

    pub fn new()-> Fixture {
        let directory = TempDir::new().expect("temp dir");
        let work = directory.path().join("work");
        let git_dir = work.join(".git");
        fs::create_dir_all(git_dir.join("objects")).expect("objects dir");
        fs::create_dir_all(git_dir.join("refs/heads")).expect("refs dir");
        fs::write(git_dir.join("HEAD"), "ref: refs/heads/change\n").expect("write HEAD");
        fs::write(git_dir.join("config"), "[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n")
            .expect("write config");
        Fixture { _directory: directory, work }
    }

    pub fn branch(&self) {
        if self.work.join(Self::TIP).exists() {
            return self.branched();
        }
        let cell = Arc::clone(BUILT.lock().expect("cache lock").entry(Snapshot::files(&self.work)).or_default());
        let mut built = false;
        let repository = cell.get_or_init(|| {
            built = true;
            self.branched();
            Snapshot::repository(&self.work)
        });
        if !built {
            repository.restore(&self.work);
        }
    }

    fn branched(&self) {
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
