use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathLine(PathBuf);

impl PathLine {
    const POINTER: &'static str = "gitdir:";

    pub fn resolved(&self, base: &Path) -> PathBuf {
        base.join(&self.0)
    }
}

impl FromStr for PathLine {
    type Err = std::convert::Infallible;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let line = text.trim();
        Ok(Self(PathBuf::from(line.strip_prefix(Self::POINTER).unwrap_or(line).trim())))
    }
}
