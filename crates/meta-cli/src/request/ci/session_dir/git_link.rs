use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug)]
pub struct GitLink(PathBuf);

impl GitLink {
    const ENTRY: &'static str = ".git";
    const PREFIX: &'static str = "gitdir:";
    const COMMON: &'static str = "commondir";
    const HEAD: &'static str = "HEAD";
    const ON_BRANCH: &'static str = "ref: refs/heads/";

    pub fn current_branch(root: &Path) -> Option<String> {
        let head = std::fs::read_to_string(Self::git_dir(root).join(Self::HEAD)).ok()?;
        head.trim().strip_prefix(Self::ON_BRANCH).map(str::to_owned)
    }

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
    use std::process::Command;

    const WORKTREE: &str = "linked";
    const IDENTITY: [&str; 4] = ["-c", "user.name=t", "-c", "user.email=t@meta.invalid"];
    const INIT: [&str; 2] = ["init", "-q"];
    const START: [&str; 5] = ["commit", "-q", "--allow-empty", "-m", "start"];
    const LINK: [&str; 4] = ["worktree", "add", "-q", WORKTREE];

    const SHOW: [&str; 2] = ["branch", "--show-current"];
    const DETACH: [&str; 3] = ["checkout", "-q", "--detach"];

    struct Git;

    impl Git {
        fn run(dir: &Path, words: &[&str]) -> String {
            let output = Command::new("git").arg("-C").arg(dir).args(IDENTITY).args(words).output().unwrap();
            String::from_utf8_lossy(&output.stdout).trim().to_owned()
        }

        fn linked() -> tempfile::TempDir {
            let main = tempfile::tempdir().unwrap();
            [INIT.as_slice(), &START, &LINK].iter().for_each(|words| drop(Self::run(main.path(), words)));
            main
        }
    }

    #[test]
    fn the_common_dir_is_the_main_git_dir_in_a_repository_and_in_a_linked_worktree() {
        let main = Git::linked();
        let expected = main.path().join(".git").canonicalize().ok();
        assert_eq!(GitLink::common_dir(main.path()), expected);
        assert_eq!(GitLink::common_dir(&main.path().join(WORKTREE)), expected);
    }

    #[test]
    fn the_current_branch_is_what_git_shows_in_a_repository_and_a_linked_worktree_and_none_when_detached() {
        let main = Git::linked();
        let linked = main.path().join(WORKTREE);
        let named = [main.path(), &linked].map(|dir| (GitLink::current_branch(dir), Some(Git::run(dir, &SHOW))));
        Git::run(main.path(), &DETACH);
        assert!(named.iter().all(|(read, shown)| read == shown && read.is_some()));
        assert_eq!(GitLink::current_branch(main.path()), None);
    }
}
