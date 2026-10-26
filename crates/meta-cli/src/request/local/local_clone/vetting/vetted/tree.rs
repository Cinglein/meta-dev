mod cargo_layout;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::request::local::byte_count::ByteCount;
use crate::request::local::local_clone::vetting::entry_kind::EntryKind;
use crate::request::local::local_clone::vetting::found::Found;
use crate::request::local::local_clone::vetting::kept::Kept;
use cargo_layout::CargoLayout;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tree(Vec<Found>);

impl Tree {
    const GIT_ENTRY: &'static str = ".git";
    const REPO_SHAPE: [(&'static str, EntryKind); 3] =
        [("HEAD", EntryKind::File), ("objects", EntryKind::Dir), ("refs", EntryKind::Dir)];
    const SIGNATURE: &'static str = "Signature: 8a477f597d28d172789f06886806bc55";
    const AUTHOR: &'static str = "created by cargo";

    pub fn new(found: Vec<Found>) -> Self {
        Self(found)
    }

    pub fn bytes(&self) -> ByteCount {
        self.0.iter().map(Found::bytes).sum()
    }

    pub fn settled(&self, tracked_links: &[PathBuf]) -> Result<(), Kept> {
        let kinds = self.kinds();
        let failures = self.0.iter().filter_map(|found| match found.kind() {
            EntryKind::Missing | EntryKind::Unreadable => Some(Kept::Unreadable),
            _ if Self::nested(found, &kinds) => Some(Kept::NestedRepo),
            _ if found.lends() => Some(Kept::Borrowed),
            EntryKind::Symlink if !tracked_links.iter().any(|link| link == found.path()) => Some(Kept::Symlink),
            EntryKind::Symlink | EntryKind::Dir | EntryKind::File => None,
        });
        failures.into_iter().next().map_or(Ok(()), Err)
    }

    pub fn cargo_target(&self, dir: &Path, tag: Option<&str>) -> Result<(), Kept> {
        tag.filter(|tag| tag.starts_with(Self::SIGNATURE) && tag.contains(Self::AUTHOR)).ok_or(Kept::Ignored)?;
        let kinds = self.kinds();
        (kinds.get(dir) == Some(&EntryKind::Dir)).then_some(()).ok_or(Kept::Ignored)?;
        let layout = CargoLayout::new(dir, &kinds);
        let inside = self.0.iter().filter_map(|found| Some((found.path().strip_prefix(dir).ok()?, found.kind())));
        inside
            .filter(|(inner, _)| !inner.as_os_str().is_empty())
            .all(|(inner, kind)| layout.owns(inner, kind))
            .then_some(())
            .ok_or(Kept::Ignored)
    }

    fn kinds(&self) -> HashMap<&Path, EntryKind> {
        self.0.iter().map(|found| (found.path(), found.kind())).collect()
    }

    fn nested(found: &Found, kinds: &HashMap<&Path, EntryKind>) -> bool {
        (found.path().file_name().is_some_and(|name| name.eq_ignore_ascii_case(Self::GIT_ENTRY))
            && !(found.path() == Path::new(Self::GIT_ENTRY) && found.kind() == EntryKind::File))
            || (found.kind() == EntryKind::Dir
                && Self::REPO_SHAPE
                    .iter()
                    .all(|(name, kind)| kinds.get(found.path().join(name).as_path()) == Some(kind)))
    }
}
