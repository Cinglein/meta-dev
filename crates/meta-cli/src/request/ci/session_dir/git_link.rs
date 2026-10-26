use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug)]
pub struct GitLink(PathBuf);

impl GitLink {
    const ENTRY: &'static str = ".git";
    const PREFIX: &'static str = "gitdir:";
    const COMMON: &'static str = "commondir";

    pub fn common_dir(root: &Path) -> Option<PathBuf> {
        let entry = root.join(Self::ENTRY);
        let linked = std::fs::read_to_string(&entry).ok().and_then(|text| text.parse::<Self>().ok());
        let git_dir = linked.map_or(entry, |link| root.join(link.0));
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
    use std::process::Command;

    const WORKTREE: &str = "linked";
    const IDENTITY: [&str; 4] = ["-c", "user.name=t", "-c", "user.email=t@meta.invalid"];
    const INIT: [&str; 2] = ["init", "-q"];
    const START: [&str; 5] = ["commit", "-q", "--allow-empty", "-m", "start"];
    const LINK: [&str; 4] = ["worktree", "add", "-q", WORKTREE];

    struct Git;

    impl Git {
        fn run(dir: &Path, words: &[&str]) {
            Command::new("git").arg("-C").arg(dir).args(IDENTITY).args(words).output().unwrap();
        }
    }

    #[test]
    fn the_common_dir_is_the_main_git_dir_in_a_repository_and_in_a_linked_worktree() {
        let main = tempfile::tempdir().unwrap();
        Git::run(main.path(), &INIT);
        Git::run(main.path(), &START);
        Git::run(main.path(), &LINK);
        let expected = main.path().join(".git").canonicalize().ok();
        assert_eq!(GitLink::common_dir(main.path()), expected);
        assert_eq!(GitLink::common_dir(&main.path().join(WORKTREE)), expected);
    }
}
