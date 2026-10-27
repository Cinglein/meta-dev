use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::request::local::local_clone::vetting::kept::Kept;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Index {
    links: Vec<PathBuf>,
}

impl Index {
    const SEPARATOR: char = '\0';
    const PATH_SEPARATOR: char = '\t';
    const CACHED: &'static str = "H";
    const MERGED: &'static str = "0";
    const PLAIN: [&'static str; 2] = ["100644", "100755"];
    const LINK: &'static str = "120000";
    const GITLINK: &'static str = "160000";

    pub fn links(&self) -> &[PathBuf] {
        &self.links
    }

    fn entry(entry: &str) -> Result<Option<PathBuf>, Kept> {
        let (shown, path) = entry.split_once(Self::PATH_SEPARATOR).ok_or(Kept::IndexFlags)?;
        match shown.split_whitespace().collect::<Vec<_>>().as_slice() {
            [Self::CACHED, Self::GITLINK, _, _] => Err(Kept::Submodules),
            [Self::CACHED, Self::LINK, _, Self::MERGED] => Ok(Some(Path::new(path).to_path_buf())),
            [Self::CACHED, mode, _, Self::MERGED] if Self::PLAIN.contains(mode) => Ok(None),
            _ => Err(Kept::IndexFlags),
        }
    }
}

impl FromStr for Index {
    type Err = Kept;

    fn from_str(listed: &str) -> Result<Self, Self::Err> {
        let entries = listed.split(Self::SEPARATOR).filter(|entry| !entry.is_empty()).map(Self::entry);
        Ok(Self { links: entries.collect::<Result<Vec<_>, _>>()?.into_iter().flatten().collect() })
    }
}
