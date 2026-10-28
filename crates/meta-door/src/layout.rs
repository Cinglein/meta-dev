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
    use std::path::Path;
    use std::process::Command;

    use super::Layout;
    use crate::Door;
    use crate::admin_file::AdminFile;

    const ENTRY: &str = ".git";
    const BRANCH: &str = "laid";
    const EMPTY: &str = "";
    const SYMBOLIC: [&str; 2] = ["symbolic-ref", "HEAD"];
    const TIP_REF: &str = "refs/heads/laid";
    const TIP_LINE: &str = "1111\n";
    const TIP: &str = "1111";
    const DETACHED: &str = "2222\n";
    const OTHER: &str = "2222";
    const HEAD: &str = "HEAD";
    const CONFIG: &str = "config";
    const ALTERNATES: &str = "objects/info/alternates";
    const CLIMB: &str = "refs/../HEAD";

    #[test]
    fn a_skeleton_is_a_repository_git_reads_on_its_branch_and_never_lays_over_a_used_dir() {
        let repository = tempfile::tempdir().unwrap();
        let git_dir = repository.path().join(ENTRY);
        Door::skeleton(&git_dir, Layout::Working, BRANCH, EMPTY).unwrap();
        let head = Command::new("git").current_dir(repository.path()).args(SYMBOLIC).output().unwrap();
        let config = Door::inspected(&git_dir).is_ok_and(|admin| admin.text(AdminFile::CommonDir).is_err());
        let again = Door::skeleton(&git_dir, Layout::Bare, BRANCH, EMPTY).is_err();
        assert!(String::from_utf8_lossy(&head.stdout).contains(BRANCH) && config && again);
    }

    #[test]
    fn a_laid_head_reads_its_branch_tip_or_detached_object_and_only_from_a_laid_repository() {
        let repository = tempfile::tempdir().unwrap();
        let git_dir = repository.path().join(ENTRY);
        Door::skeleton(&git_dir, Layout::Working, BRANCH, EMPTY).unwrap();
        let unborn = Door::laid_head(repository.path()).is_err();
        std::fs::write(git_dir.join(TIP_REF), TIP_LINE).unwrap();
        let tip = Door::laid_head(repository.path()).unwrap();
        std::fs::write(git_dir.join(HEAD), DETACHED).unwrap();
        let detached = Door::laid_head(repository.path()).unwrap();
        std::fs::write(git_dir.join(CONFIG), EMPTY).unwrap();
        let foreign = Door::laid_head(repository.path()).is_err();
        assert!(unborn && tip == TIP && detached == OTHER && foreign);
    }

    #[test]
    fn a_laid_bare_repository_takes_refs_and_alternates_and_nothing_else() {
        let bare = tempfile::tempdir().unwrap();
        Door::skeleton(bare.path(), Layout::Bare, BRANCH, EMPTY).unwrap();
        let laid = |relative: &str| Door::laid_file(bare.path(), Path::new(relative), TIP_LINE).is_ok();
        let taken = [TIP_REF, ALTERNATES].map(laid);
        let refused = [HEAD, CLIMB, CONFIG].map(laid);
        let read = std::fs::read_to_string(bare.path().join(TIP_REF)).unwrap();
        std::fs::write(bare.path().join(CONFIG), EMPTY).unwrap();
        let all = taken.iter().all(|ok| *ok) && !refused.iter().any(|ok| *ok);
        assert!(all && read == TIP_LINE && !laid(TIP_REF));
    }
}
