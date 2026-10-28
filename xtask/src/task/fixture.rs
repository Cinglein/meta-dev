use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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

    pub fn criss_cross(&self, main: [&str; 2], side: [&str; 2], head: [&str; 2]) -> &Fixture {
        let commit = |branch: &str, mark: &str, [author, committer]: [&str; 2], parents: &str| {
            format!(
                "commit {branch}\nmark :{mark}\nauthor fixture <{author}> 1700000000 +0000\ncommitter fixture <{committer}> 1700000000 +0000\ndata 8\nfixture\n{parents}\n"
            )
        };
        let stream = [
            commit("refs/remotes/origin/main", "1", main, "from refs/remotes/origin/main^0\n"),
            commit("refs/heads/change", "2", side, "from refs/heads/change^0\n"),
            commit("refs/heads/change", "3", side, "from :2\nmerge :1\n"),
            commit("refs/remotes/origin/main", "4", main, "from :2\nmerge :1\n"),
            commit("refs/heads/change", "5", head, "from :3\n"),
        ]
        .concat();
        let mut child =
            Self::git(&self.work).args(["fast-import", "--quiet"]).stdin(Stdio::piped()).spawn().expect("run git");
        child.stdin.take().expect("stdin").write_all(stream.as_bytes()).expect("write stream");
        child.wait().expect("wait git").success().then_some(()).expect("git succeeds");
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
