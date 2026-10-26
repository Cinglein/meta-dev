use std::fs::DirEntry;
use std::path::{Path, PathBuf};

use meta_schema::ByteCount;

use super::footprint::Footprint;
use super::kept::Kept;

#[derive(Debug)]
pub struct CargoTarget;

impl CargoTarget {
    const TAG: &'static str = "CACHEDIR.TAG";
    const SIGNATURE: &'static str = "Signature: 8a477f597d28d172789f06886806bc55";
    const AUTHOR: &'static str = "created by cargo";
    const LOCK: &'static str = ".cargo-lock";
    const KNOWN: [&'static str; 6] =
        ["CACHEDIR.TAG", ".rustc_info.json", ".future-incompat-report.json", "doc", "package", "tmp"];

    pub fn tagged(dir: &Path) -> bool {
        std::fs::read_to_string(dir.join(Self::TAG))
            .is_ok_and(|tag| tag.starts_with(Self::SIGNATURE) && tag.contains(Self::AUTHOR))
    }

    pub fn verified(dir: &Path) -> Result<ByteCount, Kept> {
        Self::tagged(dir).then_some(()).ok_or(Kept::Unrecognised)?;
        let bytes = Footprint::repo_free(dir)?;
        dir.read_dir()
            .into_iter()
            .flatten()
            .flatten()
            .all(|entry| Self::cargo_owned(&entry))
            .then_some(bytes)
            .ok_or(Kept::Foreign)
    }

    fn cargo_owned(entry: &DirEntry) -> bool {
        let profiled = |dir: PathBuf| dir.join(Self::LOCK).is_file();
        entry.file_name().to_str().is_some_and(|name| Self::KNOWN.contains(&name))
            || profiled(entry.path())
            || entry.path().read_dir().into_iter().flatten().flatten().any(|child| profiled(child.path()))
    }
}
