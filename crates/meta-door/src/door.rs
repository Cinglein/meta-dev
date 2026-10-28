use std::fs::{self, Metadata};
use std::io;
use std::path::{Path, PathBuf};

use crate::admin_dir::AdminDir;
use crate::admin_file::AdminFile;
use crate::change::Change;
use crate::entry::Entry;
use crate::git_dirs::GitDirs;
use crate::handle::Handle;
use crate::kind::Kind;
use crate::opening::Opening;
use crate::pair::Pair;
use crate::walked::Walked;

#[derive(Debug)]
pub struct Door;

impl Door {
    const ENTRY: &str = ".git";
    const HEAD: &str = "HEAD";
    const OBJECTS: &str = "objects";
    const LINKED: &str = "worktrees";

    pub fn read(path: impl Into<Entry>) -> io::Result<String> {
        fs::read_to_string(path.into().cleared()?)
    }

    pub fn write(path: impl Into<Entry>, contents: impl AsRef<str>) -> io::Result<()> {
        fs::write(path.into().cleared()?, contents.as_ref())
    }

    pub fn opened(path: impl Into<Entry>, opening: Opening) -> io::Result<Handle> {
        opening.options().open(path.into().cleared()?).map(Handle::new)
    }

    pub fn changed(path: impl Into<Entry>, change: Change) -> io::Result<()> {
        change.applied(&path.into().cleared()?)
    }

    pub fn paired(from: impl Into<Entry>, to: impl Into<Entry>, pair: Pair) -> io::Result<()> {
        let from = from.into();
        let to = to.into().cleared()?;
        let source = if pair == Pair::Symlink {
            let pointed = to.parent().map_or_else(|| from.path().to_path_buf(), |parent| parent.join(from.path()));
            GitDirs::outside(pointed).map(|_| from.path().to_path_buf())?
        } else {
            from.cleared()?
        };
        pair.applied(&source, &to)
    }

    pub fn listed(path: impl Into<Entry>) -> io::Result<Vec<PathBuf>> {
        fs::read_dir(path.into().cleared()?)?.map(|entry| entry.map(|entry| entry.path())).collect()
    }

    #[must_use]
    pub fn kind(path: impl Into<Entry>) -> Kind {
        Self::unfollowed(path.into()).map_or(Kind::Refused, |path| Kind::found(path.symlink_metadata()))
    }

    pub fn described(path: impl Into<Entry>) -> io::Result<Metadata> {
        Self::unfollowed(path.into())?.symlink_metadata()
    }

    fn unfollowed(entry: Entry) -> io::Result<PathBuf> {
        let parent = entry.map(|path| path.parent().map_or_else(PathBuf::new, Path::to_path_buf));
        let name = entry.path().file_name().map(ToOwned::to_owned).ok_or(io::ErrorKind::InvalidInput);
        let beside = name.map_err(io::Error::from).and_then(|name| parent.cleared().map(|parent| parent.join(name)));
        beside.or_else(|_| entry.cleared())
    }

    pub fn canonical(path: impl Into<Entry>) -> io::Result<PathBuf> {
        path.into().cleared()?.canonicalize()
    }

    pub fn inspected(directory: &Path) -> io::Result<AdminDir> {
        let real = GitDirs::resolved(directory)?;
        let repository = real
            .ancestors()
            .find(|dir| dir.join(Self::HEAD).is_file() && dir.join(Self::OBJECTS).is_dir())
            .ok_or(io::ErrorKind::NotFound)?;
        let common = GitDirs::common(repository)?;
        (real == common || real.parent() == Some(common.join(Self::LINKED).as_path()))
            .then(|| AdminDir::new(real))
            .ok_or_else(|| io::ErrorKind::PermissionDenied.into())
    }

    pub fn walked(root: impl Into<Entry>) -> io::Result<Vec<Walked>> {
        Ok(Walked::under(&root.into().cleared()?, Path::new("")))
    }

    #[must_use]
    pub fn admin_text(path: &Path) -> Option<io::Result<String>> {
        let file = AdminFile::within(path)?;
        let admin = Self::inspected(file.directory_of(path)?).ok()?;
        Some(admin.text(file))
    }

    #[must_use]
    pub fn admin_kind(path: &Path) -> Option<Kind> {
        let file = AdminFile::within(path)?;
        Some(Self::inspected(file.directory_of(path)?).ok()?.kind(file))
    }

    #[must_use]
    pub fn repository(directory: &Path) -> Option<PathBuf> {
        directory
            .ancestors()
            .find(|ancestor| ancestor.join(Self::ENTRY).symlink_metadata().is_ok())
            .map(Path::to_path_buf)
    }
}

#[cfg(feature = "skeleton")]
impl Door {
    const SKELETON: [&str; 3] = ["objects", "refs/heads", "refs/tags"];
    const UNBORN: &str = "ref: refs/heads/";
    const CONFIG: &str = "config";
    const NEWLINE: &str = "\n";
    const SYMBOLIC: &str = "ref: ";
    const REFS: &str = "refs";
    const ALTERNATES: &str = "objects/info/alternates";

    pub fn laid_file(git_dir: &Path, relative: &Path, text: &str) -> io::Result<()> {
        Self::laid(git_dir)?;
        (relative.components().all(|component| matches!(component, std::path::Component::Normal(_)))
            && (relative.starts_with(Self::REFS) || relative == Path::new(Self::ALTERNATES)))
        .then_some(())
        .ok_or(io::ErrorKind::PermissionDenied)?;
        let path = git_dir.join(relative);
        path.parent().map_or(Ok(()), fs::create_dir_all)?;
        fs::write(path, text)
    }

    fn laid(git_dir: &Path) -> io::Result<()> {
        let config = fs::read_to_string(git_dir.join(Self::CONFIG))?;
        [crate::Layout::Working, crate::Layout::Bare]
            .iter()
            .any(|layout| config.starts_with(layout.core()))
            .then_some(())
            .ok_or_else(|| io::ErrorKind::PermissionDenied.into())
    }

    pub fn laid_head(work_tree: &Path) -> io::Result<String> {
        let git_dir = work_tree.join(Self::ENTRY);
        Self::laid(&git_dir)?;
        let head = fs::read_to_string(git_dir.join(Self::HEAD))?;
        let line = head.lines().next().unwrap_or_default();
        let named = line.strip_prefix(Self::SYMBOLIC).map(|name| fs::read_to_string(git_dir.join(name)));
        let object = named.unwrap_or_else(|| Ok(line.to_owned()))?;
        Ok(object.lines().next().unwrap_or_default().to_owned())
    }

    pub fn skeleton(git_dir: &Path, layout: crate::Layout, branch: &str, config: &str) -> io::Result<()> {
        git_dir
            .read_dir()
            .map_or(true, |mut entries| entries.next().is_none())
            .then_some(())
            .ok_or(io::ErrorKind::AlreadyExists)?;
        GitDirs::outside(git_dir.to_path_buf())?;
        Self::SKELETON.iter().try_for_each(|dir| fs::create_dir_all(git_dir.join(dir)))?;
        fs::write(git_dir.join(Self::HEAD), [Self::UNBORN, branch, Self::NEWLINE].concat())?;
        fs::write(git_dir.join(Self::CONFIG), [layout.core(), config].concat())
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
    const PLANTED: &str = "meta-local/planted";
    const CLIMB: &str = "missing/../.git/refs/heads/climbed";
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
    fn a_bare_repository_is_refused_by_name_and_neither_a_link_nor_a_copy_may_reach_into_it() {
        let bare = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        assert!(Command::new("git").current_dir(bare.path()).args(BARE).status().unwrap().success());
        let refused = Door::write(bare.path().join(STORED_REFS), EMPTY).unwrap_err().to_string();
        let linked = Door::paired(bare.path().join(HEAD), outside.path().join(HEAD), Pair::Symlink);
        let copied = Door::paired(bare.path().join(HEAD), outside.path().join(WORK), Pair::Copy);
        assert!(refused.contains(REFUSED) && linked.is_err() && copied.is_err(), "{refused}");
    }

    #[test]
    fn a_planted_link_or_a_climb_out_of_a_missing_dir_never_reaches_the_git_dir() {
        let repository = tempfile::tempdir().unwrap();
        assert!(Command::new("git").current_dir(repository.path()).args(INIT).status().unwrap().success());
        let store = MetaStore::located(repository.path()).unwrap();
        let refs = store.common().join(STORED_REFS);
        Door::changed(store.entry(AREA), Change::DirAll).unwrap();
        let linked = Door::paired(&refs, store.entry(TRAIL), Pair::Symlink);
        std::os::unix::fs::symlink(&refs, store.entry(PLANTED).path()).unwrap();
        let through = Door::write(store.entry(PLANTED), EMPTY);
        let climbed = Door::changed(repository.path().join(CLIMB), Change::DirAll);
        assert!(linked.is_err() && through.is_err() && climbed.is_err() && !refs.exists());
    }
}
