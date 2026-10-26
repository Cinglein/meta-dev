use std::collections::HashMap;
use std::path::{Component, Path};

use crate::request::local::local_clone::vetting::entry_kind::EntryKind;

#[derive(Clone, Copy, Debug)]
pub struct CargoLayout<'tree> {
    dir: &'tree Path,
    kinds: &'tree HashMap<&'tree Path, EntryKind>,
}

impl<'tree> CargoLayout<'tree> {
    const TOP_FILES: [&'static str; 3] = ["CACHEDIR.TAG", ".rustc_info.json", ".future-incompat-report.json"];
    const FREE_DIRS: [&'static str; 3] = ["doc", "package", "tmp"];
    const PROFILE_DIRS: [&'static str; 5] = [".fingerprint", "build", "deps", "examples", "incremental"];
    const LOCK: &'static str = ".cargo-lock";
    const ARTIFACTS: [&'static str; 11] =
        ["d", "rlib", "rmeta", "so", "dylib", "dll", "a", "lib", "exe", "pdb", "wasm"];
    const DEBUG_SYMBOLS: &'static str = ".dSYM";
    const EXTENSION: char = '.';

    pub fn new(dir: &'tree Path, kinds: &'tree HashMap<&'tree Path, EntryKind>) -> Self {
        Self { dir, kinds }
    }

    pub fn owns(&self, inner: &Path, kind: EntryKind) -> bool {
        let parts: Vec<&str> = inner.components().map(Component::as_os_str).filter_map(|part| part.to_str()).collect();
        match parts.as_slice() {
            [name] if kind == EntryKind::File => Self::TOP_FILES.contains(name),
            [name] => {
                kind == EntryKind::Dir && (Self::FREE_DIRS.contains(name) || self.profile(&[name]) || self.triple(name))
            }
            [free, ..] if Self::FREE_DIRS.contains(free) => true,
            [profile, rest @ ..] if self.profile(&[profile]) => Self::in_profile(rest, kind),
            [_triple, _profile, rest @ ..] => Self::in_profile(rest, kind),
            [] => false,
        }
    }

    fn profile(&self, parts: &[&str]) -> bool {
        let dir = parts.iter().fold(self.dir.to_path_buf(), |dir, part| dir.join(part));
        self.kinds.get(dir.join(Self::LOCK).as_path()) == Some(&EntryKind::File)
    }

    fn triple(&self, name: &str) -> bool {
        let triple = self.dir.join(name);
        let children: Vec<&&Path> = self.kinds.keys().filter(|path| path.parent() == Some(triple.as_path())).collect();
        !children.is_empty()
            && children.iter().all(|child| {
                child
                    .file_name()
                    .and_then(|profile| profile.to_str())
                    .is_some_and(|profile| self.profile(&[name, profile]))
            })
    }

    fn in_profile(rest: &[&str], kind: EntryKind) -> bool {
        match rest {
            [] => kind == EntryKind::Dir,
            [lock] if *lock == Self::LOCK => kind == EntryKind::File,
            [dir] if kind == EntryKind::Dir => Self::PROFILE_DIRS.contains(dir) || dir.ends_with(Self::DEBUG_SYMBOLS),
            [file] => kind == EntryKind::File && Self::artifact(file),
            [dir, ..] => Self::PROFILE_DIRS.contains(dir) || dir.ends_with(Self::DEBUG_SYMBOLS),
        }
    }

    fn artifact(name: &str) -> bool {
        name.rsplit_once(Self::EXTENSION).is_none_or(|(_, extension)| Self::ARTIFACTS.contains(&extension))
    }
}
