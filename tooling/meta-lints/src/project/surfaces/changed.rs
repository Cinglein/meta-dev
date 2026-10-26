use std::env;
use std::fmt;
use std::fs::OpenOptions;
use std::io::Write;

use crate::failure::Failure;
use crate::prior::ChangedPath;

#[derive(Clone, Debug)]
pub struct Changed(Vec<ChangedPath>);

impl Changed {
    const SUMMARY: &str = "GITHUB_STEP_SUMMARY";

    pub fn new(paths: Vec<ChangedPath>) -> Changed {
        Changed(paths)
    }

    pub fn announce(&self) -> Result<(), Failure> {
        if self.0.is_empty() {
            Ok(())
        } else {
            print!("{self}");
            env::var_os(Self::SUMMARY)
                .map_or(Ok(()), |summary| {
                    OpenOptions::new().append(true).create(true).open(summary)?.write_all(self.to_string().as_bytes())
                })
                .map_err(Failure::Summary)
        }
    }
}

impl fmt::Display for Changed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "\n!! enforcement surfaces changed since the merge base:")?;
        self.0.iter().try_for_each(|path| writeln!(formatter, "- {path}"))
    }
}
