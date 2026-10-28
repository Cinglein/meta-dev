#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    Bare,
    Working,
}

impl Layout {
    pub(crate) fn core(self) -> &'static str {
        match self {
            Layout::Bare => {
                "[core]\n\trepositoryformatversion = 1\n\tbare = true\n[extensions]\n\trefStorage = files\n"
            }
            Layout::Working => {
                "[core]\n\trepositoryformatversion = 1\n\tbare = false\n[extensions]\n\trefStorage = files\n"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::Layout;
    use crate::Door;
    use crate::admin_file::AdminFile;

    const ENTRY: &str = ".git";
    const BRANCH: &str = "laid";
    const EMPTY: &str = "";
    const SYMBOLIC: [&str; 2] = ["symbolic-ref", "HEAD"];

    #[test]
    fn a_skeleton_is_a_repository_git_reads_on_its_branch_and_never_lays_over_a_used_dir() {
        let repository = tempfile::tempdir().unwrap();
        let git_dir = repository.path().join(ENTRY);
        Door::skeleton(&git_dir, Layout::Working, BRANCH, EMPTY).unwrap();
        let head = Command::new("git").current_dir(repository.path()).args(SYMBOLIC).output().unwrap();
        let config = Door::inspected(&git_dir, AdminFile::CommonDir).is_err();
        let again = Door::skeleton(&git_dir, Layout::Bare, BRANCH, EMPTY).is_err();
        assert!(String::from_utf8_lossy(&head.stdout).contains(BRANCH) && config && again);
    }
}
