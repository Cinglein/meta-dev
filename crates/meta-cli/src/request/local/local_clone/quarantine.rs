mod stamp;

use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub use stamp::Stamp;

#[derive(Clone, Copy, Debug)]
pub struct Quarantine;

impl Quarantine {
    const PREFIX: &'static str = ".gc-";
    const SEPARATOR: &'static str = "-";

    pub fn named(path: &Path, stamp: &Stamp) -> PathBuf {
        let mut name = OsString::from(Self::PREFIX);
        name.push(path.file_name().unwrap_or_default());
        name.push(Self::SEPARATOR);
        name.push(stamp);
        path.with_file_name(name)
    }

    pub fn holds(path: &Path) -> bool {
        path.file_name().is_some_and(|name| name.to_string_lossy().starts_with(Self::PREFIX))
    }

    pub fn origin(path: &Path) -> Option<PathBuf> {
        let name = path.file_name()?.to_string_lossy().into_owned();
        let (original, _) = name.strip_prefix(Self::PREFIX)?.rsplit_once(Self::SEPARATOR)?;
        Some(path.with_file_name(original))
    }
}
