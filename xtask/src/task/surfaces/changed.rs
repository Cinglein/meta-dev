use std::fmt;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::task::failure::Failure;

#[derive(Debug)]
pub struct Changed(Vec<PathBuf>);

impl Changed {
    pub fn new(paths: Vec<PathBuf>) -> Changed {
        Changed(paths)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn announce(&self, summary: Option<&Path>) -> Result<(), Failure> {
        print!("{self}");
        summary
            .filter(|_| !self.0.is_empty())
            .map_or(Ok(()), |summary| {
                OpenOptions::new().append(true).create(true).open(summary)?.write_all(self.to_string().as_bytes())
            })
            .map_err(Failure::Summary)
    }
}

impl fmt::Display for Changed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            Ok(())
        } else {
            writeln!(formatter, "\n!! enforcement surfaces changed since the merge base:")?;
            self.0.iter().try_for_each(|path| writeln!(formatter, "- {}", path.display()))
        }
    }
}
