use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

use crate::request::local::byte_count::ByteCount;
use crate::request::local::local_clone::vetting::entry_kind::EntryKind;
use crate::request::local::local_clone::vetting::found::Found;
use crate::request::local::local_clone::vetting::kept::Kept;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tree(Vec<Found>);

impl Tree {
    const GIT_ENTRY: &'static str = ".git";
    const ALTERNATES: [&'static str; 3] = ["alternates", "info", "objects"];
    const REPO_SHAPE: [(&'static str, EntryKind); 3] =
        [("HEAD", EntryKind::File), ("objects", EntryKind::Dir), ("refs", EntryKind::Dir)];
    const SIGNATURE: &'static str = "Signature: 8a477f597d28d172789f06886806bc55";
    const AUTHOR: &'static str = "created by cargo";
    const LOCK: &'static str = ".cargo-lock";
    const KNOWN: [&'static str; 6] =
        ["CACHEDIR.TAG", ".rustc_info.json", ".future-incompat-report.json", "doc", "package", "tmp"];

    pub fn new(found: Vec<Found>) -> Self {
        Self(found)
    }

    pub fn bytes(&self) -> ByteCount {
        self.0.iter().map(Found::bytes).sum()
    }

    pub fn settled(&self, tracked_links: &[PathBuf]) -> Result<(), Kept> {
        let kinds: HashMap<&Path, EntryKind> = self.0.iter().map(|found| (found.path(), found.kind())).collect();
        let failures = self.0.iter().filter_map(|found| match found.kind() {
            EntryKind::Missing | EntryKind::Unreadable => Some(Kept::Unreadable),
            _ if Self::nested(found, &kinds) => Some(Kept::NestedRepo),
            _ if Self::lending(found.path()) => Some(Kept::Borrowed),
            EntryKind::Symlink if !tracked_links.iter().any(|link| link == found.path()) => Some(Kept::Symlink),
            EntryKind::Symlink | EntryKind::Dir | EntryKind::File => None,
        });
        failures.into_iter().next().map_or(Ok(()), Err)
    }

    pub fn cargo_target(&self, dir: &Path, tag: Option<&str>) -> Result<(), Kept> {
        tag.filter(|tag| tag.starts_with(Self::SIGNATURE) && tag.contains(Self::AUTHOR)).ok_or(Kept::Ignored)?;
        self.0.iter().find(|found| found.path() == dir && found.kind() == EntryKind::Dir).ok_or(Kept::Ignored)?;
        self.0
            .iter()
            .filter(|found| found.path().parent() == Some(dir))
            .all(|child| self.cargo_owned(child))
            .then_some(())
            .ok_or(Kept::Ignored)
    }

    fn cargo_owned(&self, child: &Found) -> bool {
        child.path().file_name().and_then(OsStr::to_str).is_some_and(|name| Self::KNOWN.contains(&name))
            || self.0.iter().any(|lock| Self::locks(lock, child.path()))
    }

    fn nested(found: &Found, kinds: &HashMap<&Path, EntryKind>) -> bool {
        (found.path().file_name().is_some_and(|name| name.eq_ignore_ascii_case(Self::GIT_ENTRY))
            && !(found.path() == Path::new(Self::GIT_ENTRY) && found.kind() == EntryKind::File))
            || (found.kind() == EntryKind::Dir
                && Self::REPO_SHAPE
                    .iter()
                    .all(|(name, kind)| kinds.get(found.path().join(name).as_path()) == Some(kind)))
    }

    fn lending(path: &Path) -> bool {
        let tail: Vec<&OsStr> =
            path.components().rev().take(Self::ALTERNATES.len()).map(Component::as_os_str).collect();
        tail.len() == Self::ALTERNATES.len()
            && tail.iter().zip(Self::ALTERNATES).all(|(part, name)| part.eq_ignore_ascii_case(name))
    }

    fn locks(lock: &Found, child: &Path) -> bool {
        lock.path().file_name() == Some(OsStr::new(Self::LOCK))
            && lock.kind() == EntryKind::File
            && (lock.path().parent() == Some(child) || lock.path().parent().and_then(Path::parent) == Some(child))
    }
}
