use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use tempfile::TempDir;

mod branched;
pub use branched::Branched;

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
        Branched::Tracked.applied(&self.work);
    }

    pub fn imported(&self, branched: Branched) -> &Fixture {
        let parent = if self.work.join(".git/refs/heads/change").exists() { "from refs/heads/change^0\n" } else { "" };
        let files = Self::files(&self.work, Path::new(""));
        let stream = format!(
            "commit refs/heads/change\ncommitter fixture <fixture@invalid> 1700000000 +0000\ndata 8\nfixture\n{parent}deleteall\n{files}\n"
        );
        let mut child = Self::git(&self.work)
            .args(["-c", "fastimport.unpackLimit=0", "fast-import", "--quiet"])
            .stdin(Stdio::piped())
            .spawn()
            .expect("run git");
        child.stdin.take().expect("stdin").write_all(stream.as_bytes()).expect("write stream");
        child.wait().expect("wait git").success().then_some(()).expect("git succeeds");
        branched.applied(&self.work);
        self
    }

    fn files(root: &Path, under: &Path) -> String {
        let entries = fs::read_dir(root.join(under)).expect("read dir");
        let named = entries.map(|entry| under.join(entry.expect("dir entry").file_name()));
        named.filter(|path| path != Path::new(".git")).fold(String::new(), |mut files, path| {
            let listed = if root.join(&path).is_dir() {
                Self::files(root, &path)
            } else {
                let text = fs::read_to_string(root.join(&path)).expect("read file");
                let shown = path.display().to_string();
                ["M 100644 inline ", &shown, "\ndata ", &text.len().to_string(), "\n", &text, "\n"].concat()
            };
            files.push_str(&listed);
            files
        })
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
