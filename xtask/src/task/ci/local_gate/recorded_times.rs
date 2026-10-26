use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct RecordedTimes(PathBuf);

impl RecordedTimes {
    pub fn new(path: PathBuf) -> RecordedTimes {
        RecordedTimes(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl fmt::Display for RecordedTimes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&fs::read_to_string(&self.0).unwrap_or_default())
    }
}
