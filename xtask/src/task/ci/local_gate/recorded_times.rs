use meta_door::Door;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::task::ci::local_gate::PassedSteps;
use crate::task::failure::Failure;

#[derive(Debug)]
pub struct RecordedTimes(PathBuf);

impl RecordedTimes {
    pub fn new(path: PathBuf) -> RecordedTimes {
        RecordedTimes(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn passed(&self) -> Result<PassedSteps, Failure> {
        Door::read(&self.0).unwrap_or_default().parse()
    }
}

impl fmt::Display for RecordedTimes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&Door::read(&self.0).unwrap_or_default())
    }
}
