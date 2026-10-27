use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug)]
pub struct GitLink(PathBuf);

impl GitLink {
    const ENTRY: &'static str = ".git";
    const PREFIX: &'static str = "gitdir:";
    const COMMON: &'static str = "commondir";

    fn git_dir(root: &Path) -> PathBuf {
        let entry = root.join(Self::ENTRY);
        let linked = std::fs::read_to_string(&entry).ok().and_then(|text| text.parse::<Self>().ok());
        linked.map_or(entry, |link| root.join(link.0))
    }

    pub fn common_dir(root: &Path) -> Option<PathBuf> {
        let git_dir = Self::git_dir(root);
        let common = std::fs::read_to_string(git_dir.join(Self::COMMON))
            .ok()
            .and_then(|text| text.parse::<Self>().ok())
            .map_or_else(|| git_dir.clone(), |common| git_dir.join(common.0));
        common.canonicalize().ok()
    }
}

impl FromStr for GitLink {
    type Err = ();

    fn from_str(text: &str) -> Result<Self, ()> {
        let path = text.trim();
        let linked = path.strip_prefix(Self::PREFIX).map_or(path, str::trim);
        Some(linked).filter(|linked| !linked.is_empty()).map(|linked| Self(PathBuf::from(linked))).ok_or(())
    }
}

#[cfg(test)]
mod tests {
    use super::GitLink;
    use std::path::Path;

    const WORKTREE: &str = "linked";
    const ENTRY: &str = ".git";
    const HEAD: &str = "HEAD";
    const COMMON: &str = "commondir";
    const ADMIN: &str = ".git/worktrees/linked";
    const MAIN_HEAD: &str = "ref: refs/heads/main\n";
    const LINKED_HEAD: &str = "ref: refs/heads/linked\n";
    const BACK_TO_COMMON: &str = "../..\n";

    struct Layout;

    impl Layout {
        fn linked(main: &Path) {
            let admin = main.join(ADMIN);
            std::fs::create_dir_all(&admin).unwrap();
            std::fs::create_dir_all(main.join(WORKTREE)).unwrap();
            std::fs::write(main.join(ENTRY).join(HEAD), MAIN_HEAD).unwrap();
            std::fs::write(admin.join(HEAD), LINKED_HEAD).unwrap();
            std::fs::write(admin.join(COMMON), BACK_TO_COMMON).unwrap();
            let link = format!("{} {}\n", GitLink::PREFIX, admin.display());
            std::fs::write(main.join(WORKTREE).join(ENTRY), link).unwrap();
        }
    }

    #[test]
    fn the_common_dir_is_the_main_git_dir_in_a_repository_and_in_a_linked_worktree() {
        let main = tempfile::tempdir().unwrap();
        Layout::linked(main.path());
        let expected = main.path().join(ENTRY).canonicalize().ok();
        assert_eq!(GitLink::common_dir(main.path()), expected);
        assert_eq!(GitLink::common_dir(&main.path().join(WORKTREE)), expected);
    }
}
