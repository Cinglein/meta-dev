use std::io::Write as _;
use std::path::Path;
use std::process::Command;

pub enum Skeleton {
    Bare,
    Working,
}

impl Skeleton {
    const INIT: [&'static str; 4] = ["init", "-q", "--template=", "-b"];
    const MAINTENANCE: &'static str = "[maintenance]\n\tauto = false\n";

    pub fn lay(self, dir: &Path, branch: &str) {
        let (git_dir, shape) = match self {
            Self::Bare => (dir.to_path_buf(), "--bare"),
            Self::Working => (dir.join(".git"), "-q"),
        };
        let init = Command::new("git").arg("-C").arg(dir).args(Self::INIT).args([branch, shape, "."]).status();
        assert!(init.unwrap().success());
        let config = std::fs::OpenOptions::new().append(true).open(git_dir.join("config"));
        config.unwrap().write_all(Self::MAINTENANCE.as_bytes()).unwrap();
    }
}
