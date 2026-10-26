use std::path::Path;

pub struct Skeleton;

impl Skeleton {
    const OBJECTS: &'static str = "objects";
    const REFS: &'static str = "refs";
    const HEAD: &'static str = "HEAD";
    const CONFIG: &'static str = "config";
    const BARE: &'static str = "[core]\n\trepositoryformatversion = 0\n\tbare = true\n[maintenance]\n\tauto = false\n";
    const WORKING: &'static str =
        "[core]\n\trepositoryformatversion = 0\n\tbare = false\n[maintenance]\n\tauto = false\n";

    pub fn bare(dir: &Path, branch: &str) {
        Self::lay(dir, branch, Self::BARE);
    }

    pub fn working(dir: &Path, branch: &str) {
        Self::lay(&dir.join(".git"), branch, Self::WORKING);
    }

    fn lay(git_dir: &Path, branch: &str, config: &str) {
        std::fs::create_dir_all(git_dir.join(Self::OBJECTS)).unwrap();
        std::fs::create_dir_all(git_dir.join(Self::REFS)).unwrap();
        std::fs::write(git_dir.join(Self::HEAD), format!("ref: refs/heads/{branch}\n")).unwrap();
        std::fs::write(git_dir.join(Self::CONFIG), config).unwrap();
    }
}
