use std::fmt;
use std::io::{ErrorKind, Write as _};
use std::path::{Path, PathBuf};

use crate::argv::Argv;
use crate::golden_failure::GoldenFailure;
use crate::recording::Recording;

#[derive(Clone, Debug)]
pub struct Claim {
    key: PathBuf,
    index: usize,
}

impl Claim {
    pub fn first(key: PathBuf) -> Self {
        Self { key, index: 0 }
    }

    pub fn following(&self) -> Self {
        Self { key: self.key.clone(), index: self.index + 1 }
    }

    pub fn answer<'answers>(&self, answers: &'answers [Recording]) -> Option<&'answers Recording> {
        answers.get(self.index)
    }

    pub fn taken(&self, trail: &Path, asked: &Argv) -> Result<bool, GoldenFailure> {
        let path = trail.join(self.to_string());
        let unwritable = |source| GoldenFailure::Unreadable { path: path.clone(), source };
        match std::fs::OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => (&file).write_all(asked.to_string().as_bytes()).map(|()| true).map_err(unwritable),
            Err(source) if source.kind() == ErrorKind::AlreadyExists => Ok(false),
            Err(source) => Err(unwritable(source)),
        }
    }
}

impl fmt::Display for Claim {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}", self.key.display(), self.index)
    }
}
