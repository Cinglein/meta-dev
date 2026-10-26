use std::ffi::OsStr;
use std::fs::DirEntry;
use std::path::Path;

use meta_schema::ByteCount;

use super::kept::Kept;
use crate::request::cli_failure::Message;

#[derive(Debug)]
pub struct Footprint;

impl Footprint {
    const GIT_ENTRY: &'static str = ".git";

    pub fn measured(path: &Path) -> ByteCount {
        match path.symlink_metadata() {
            Ok(entry) if entry.is_dir() => {
                std::fs::read_dir(path).into_iter().flatten().flatten().map(|child| Self::measured(&child.path())).sum()
            }
            Ok(entry) => ByteCount::new(entry.len()),
            Err(_) => ByteCount::default(),
        }
    }

    pub fn repo_free(path: &Path) -> Result<ByteCount, Kept> {
        let entry = path.symlink_metadata().map_err(|failure| Kept::Uninspected(Message::new(failure)))?;
        match (path.file_name() == Some(OsStr::new(Self::GIT_ENTRY)), entry.is_dir()) {
            (true, _) => Err(Kept::HoldsRepo),
            (false, true) => {
                path.read_dir().map_err(|failure| Kept::Uninspected(Message::new(failure)))?.map(Self::child_free).sum()
            }
            (false, false) => Ok(ByteCount::new(entry.len())),
        }
    }

    fn child_free(child: std::io::Result<DirEntry>) -> Result<ByteCount, Kept> {
        Self::repo_free(&child.map_err(|failure| Kept::Uninspected(Message::new(failure)))?.path())
    }
}
