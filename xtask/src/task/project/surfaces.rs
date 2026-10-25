use std::env;
use std::fmt;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use meta_lints::Revision;
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct Surfaces(Vec<PathBuf>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Changed(Vec<String>);

impl Surfaces {
    pub fn union(&self, prior: Surfaces) -> Surfaces {
        Surfaces(self.0.iter().cloned().chain(prior.0).collect())
    }

    pub fn changed(&self, revision: &Revision) -> Result<Changed, meta_lints::Failure> {
        let paths = revision.changed_paths()?;
        Ok(Changed(
            paths
                .into_iter()
                .filter(|path| self.0.iter().any(|surface| Path::new(path).starts_with(surface)))
                .collect(),
        ))
    }
}

impl Changed {
    const SUMMARY: &str = "GITHUB_STEP_SUMMARY";

    pub fn announce(&self) -> io::Result<()> {
        if self.0.is_empty() {
            return Ok(());
        }
        print!("{self}");
        env::var_os(Self::SUMMARY).map_or(Ok(()), |summary| {
            OpenOptions::new().append(true).create(true).open(summary)?.write_all(self.to_string().as_bytes())
        })
    }
}

impl fmt::Display for Changed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "\n!! enforcement surfaces changed since the merge base:")?;
        self.0.iter().try_for_each(|path| writeln!(formatter, "- {path}"))
    }
}
