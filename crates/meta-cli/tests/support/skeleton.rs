use std::fmt;
use std::path::Path;

pub enum Skeleton {
    Bare,
    Working,
}

struct Head<'name>(&'name str);

impl fmt::Display for Head<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "ref: refs/heads/{}", self.0)
    }
}

impl Skeleton {
    const BARE: &'static str = "[core]\n\trepositoryformatversion = 0\n\tbare = true\n[maintenance]\n\tauto = false\n";
    const WORKING: &'static str =
        "[core]\n\trepositoryformatversion = 0\n\tbare = false\n[maintenance]\n\tauto = false\n";

    pub fn lay(self, dir: &Path, branch: &str) {
        let (git_dir, config) = match self {
            Self::Bare => (dir.to_path_buf(), Self::BARE),
            Self::Working => (dir.join(".git"), Self::WORKING),
        };
        std::fs::create_dir_all(git_dir.join("objects")).unwrap();
        std::fs::create_dir_all(git_dir.join("refs")).unwrap();
        std::fs::write(git_dir.join("HEAD"), Head(branch).to_string()).unwrap();
        std::fs::write(git_dir.join("config"), config).unwrap();
    }
}
