use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::project::Project;
use tempfile::TempDir;

#[derive(Debug)]
pub struct Fixture {
    _directory: TempDir,
    work: PathBuf,
}

impl Fixture {
    pub fn new() -> Fixture {
        let directory = TempDir::new().expect("temp dir");
        let origin = directory.path().join("origin.git");
        let work = directory.path().join("work");
        Self::ran(Self::git(directory.path()).args(["init", "--quiet", "--bare"]).arg(&origin));
        Self::ran(Self::git(directory.path()).args(["init", "--quiet", "--initial-branch", "main"]).arg(&work));
        Self::ran(Self::git(&work).args(["remote", "add", "origin"]).arg(&origin));
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
        Self::ran(Self::git(&self.work).args(["push", "--quiet", "origin", "main"]));
        Self::ran(Self::git(&self.work).args(["remote", "set-head", "origin", "main"]));
        Self::ran(Self::git(&self.work).args(["switch", "--quiet", "--create", "change"]));
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

    pub fn project(&self) -> Project {
        Project::locate(&self.work).expect("configured project")
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
