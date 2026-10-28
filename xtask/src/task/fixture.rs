use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, LazyLock, Mutex, OnceLock};

use tempfile::TempDir;

use crate::task::snapshot::Snapshot;

static BUILT: LazyLock<Mutex<BuiltRepositories>> = LazyLock::new(Mutex::default);

type BuiltRepositories = HashMap<(Snapshot, String), Arc<OnceLock<Snapshot>>>;

#[derive(Debug)]
pub struct Fixture {
    _directory: TempDir,
    work: PathBuf,
}

impl Fixture {
    const TIP: &str = ".git/refs/heads/change";

    pub fn new() -> Fixture {
        let directory = TempDir::new().expect("temp dir");
        let work = directory.path().join("work");
        Self::laid(&work.join(".git"), "false");
        Fixture { _directory: directory, work }
    }

    fn laid(git_dir: &Path, bare: &str) {
        fs::create_dir_all(git_dir.join("objects")).expect("objects dir");
        fs::create_dir_all(git_dir.join("refs/heads")).expect("refs dir");
        fs::write(git_dir.join("HEAD"), "ref: refs/heads/change\n").expect("write HEAD");
        fs::write(git_dir.join("config"), Self::config(bare)).expect("write config");
    }

    fn config(bare: &str) -> String {
        format!("[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = {bare}\n")
    }

    pub fn branch(&self) {
        if self.work.join(Self::TIP).exists() {
            return self.branched();
        }
        let config = fs::read_to_string(self.work.join(".git/config")).expect("read config");
        let key = (Snapshot::files(&self.work), config);
        let cell = Arc::clone(BUILT.lock().expect("cache lock").entry(key).or_default());
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
        Self::laid(&bare, "true");
        self.pointed(&bare);
        other.pointed(&bare);
        bare
    }

    pub fn pointed(&self, origin: &Path) -> &Fixture {
        let config = format!("{}[remote \"origin\"]\n\turl = {}\n", Self::config("false"), origin.display());
        fs::write(self.work.join(".git/config"), config).expect("write remote config");
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
        let stream = format!(
            "commit refs/heads/first\nmark :1\n{}from refs/heads/change\n\ncommit refs/heads/change\nmark :2\n{}\n\
             commit refs/heads/change\n{}merge :1\n\ncommit refs/remotes/origin/main\n{}from :2\nmerge :1\n\n\
             reset refs/heads/first\n\n",
            Self::stamped(main, "2000000000"),
            Self::stamped(side, "2000000001"),
            Self::stamped(side, "2000000002"),
            Self::stamped(main, "2000000003"),
        );
        let mut child = Self::git(&self.work)
            .args(["fast-import", "--quiet"])
            .stdin(Stdio::piped())
            .spawn()
            .expect("spawn fast-import");
        child.stdin.take().expect("fast-import stdin").write_all(stream.as_bytes()).expect("write stream");
        child.wait().expect("wait for fast-import").success().then_some(()).expect("fast-import succeeds");
        self
    }

    fn stamped([author, committer]: [&str; 2], time: &str) -> String {
        format!(
            "author fixture <{author}> {time} +0000\ncommitter fixture <{committer}> {time} +0000\ndata 8\nfixture\n"
        )
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
