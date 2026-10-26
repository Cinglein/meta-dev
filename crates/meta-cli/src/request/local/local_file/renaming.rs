use std::fmt;
use std::io;
use std::path::Path;

#[derive(Debug)]
pub struct Renaming<'path> {
    target: &'path Path,
    failure: io::Error,
}

impl<'path> Renaming<'path> {
    pub fn failed_onto(target: &'path Path, failure: io::Error) -> Self {
        Self { target, failure }
    }
}

impl fmt::Display for Renaming<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "renaming to {}: {}", self.target.display(), self.failure)
    }
}
