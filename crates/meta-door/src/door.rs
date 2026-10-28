use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use crate::change::Change;
use crate::entry::Entry;
use crate::opening::Opening;
use crate::pair::Pair;

#[derive(Debug)]
pub struct Door;

impl Door {
    pub fn read(path: impl Into<Entry>) -> io::Result<String> {
        fs::read_to_string(path.into().cleared()?)
    }

    pub fn write(path: impl Into<Entry>, contents: impl AsRef<str>) -> io::Result<()> {
        fs::write(path.into().cleared()?, contents.as_ref())
    }

    pub fn opened(path: impl Into<Entry>, opening: Opening) -> io::Result<File> {
        opening.options().open(path.into().cleared()?)
    }

    pub fn changed(path: impl Into<Entry>, change: Change) -> io::Result<()> {
        change.applied(&path.into().cleared()?)
    }

    pub fn paired(from: impl Into<Entry>, to: impl Into<Entry>, pair: Pair) -> io::Result<()> {
        let from = from.into();
        let source = if pair == Pair::Symlink { from.path().to_path_buf() } else { from.cleared()? };
        pair.applied(&source, &to.into().cleared()?)
    }

    pub fn listed(path: impl Into<Entry>) -> io::Result<Vec<PathBuf>> {
        fs::read_dir(path.into().cleared()?)?.map(|entry| entry.map(|entry| entry.path())).collect()
    }

    pub fn inspected(admin_file: &Path) -> io::Result<String> {
        fs::read_to_string(admin_file)
    }
}

#[cfg(feature = "dangling-pause")]
impl Door {
    const PAUSES: [&str; 4] = ["MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD", "REBASE_HEAD"];
    const LINE_END: &str = "\n";
    const GIT_DIR: [&str; 2] = ["rev-parse", "--absolute-git-dir"];

    pub fn dangling_pause(worktree: &Path, pause: &str, commit: &str) -> io::Result<()> {
        Self::PAUSES.contains(&pause).then_some(()).ok_or(io::ErrorKind::InvalidInput)?;
        let asked = std::process::Command::new("git").arg("-C").arg(worktree).args(Self::GIT_DIR).output()?;
        let text = String::from_utf8_lossy(&asked.stdout);
        let git = text.lines().next().ok_or(io::ErrorKind::NotFound)?;
        fs::write(Path::new(git).join(pause), [commit, Self::LINE_END].concat())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use super::Door;
    use crate::{Change, Entry, MetaStore, Pair};

    const INIT: [&str; 2] = ["init", "-q"];
    const BARE: [&str; 3] = ["init", "-q", "--bare"];
    const AREA: &str = "meta-local";
    const TRAIL: &str = "meta-local/trail";
    const ESCAPED: &str = "meta-local/../HEAD";
    const STORED_REFS: &str = "refs/heads/main";
    const REFS: &str = ".git/refs/heads/main";
    const HEAD: &str = "HEAD";
    const WORK: &str = "work";
    const UP: &str = "..";
    const EMPTY: &str = "";
    const REFUSED: &str = "is inside the git dir";

    #[test]
    fn the_door_refuses_the_git_dir_and_lets_meta_store_files_and_the_work_tree_through() {
        let repository = tempfile::tempdir().unwrap();
        assert!(Command::new("git").current_dir(repository.path()).args(INIT).status().unwrap().success());
        let store = MetaStore::located(repository.path()).unwrap();
        let made = |entry: Entry| Door::changed(entry, Change::DirAll);
        let parent = |path: &Path| path.join(UP);
        let refused = [store.entry(TRAIL).map(parent), store.entry(ESCAPED), store.entry(STORED_REFS)];
        assert!(refused.into_iter().all(|entry| made(entry).is_err()) && made(store.entry(AREA)).is_ok());
        let work =
            Door::write(repository.path().join(WORK), EMPTY).and_then(|()| Door::write(store.entry(TRAIL), EMPTY));
        assert!(Door::write(repository.path().join(REFS), EMPTY).is_err() && work.is_ok());
    }

    #[test]
    fn a_bare_repository_is_refused_by_name_and_a_link_may_point_into_it_but_a_copy_may_not_read_it() {
        let bare = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        assert!(Command::new("git").current_dir(bare.path()).args(BARE).status().unwrap().success());
        let refused = Door::write(bare.path().join(STORED_REFS), EMPTY).unwrap_err().to_string();
        let linked = Door::paired(bare.path().join(HEAD), outside.path().join(HEAD), Pair::Symlink);
        let copied = Door::paired(bare.path().join(HEAD), outside.path().join(WORK), Pair::Copy);
        assert!(refused.contains(REFUSED) && linked.is_ok() && copied.is_err(), "{refused}");
    }
}
