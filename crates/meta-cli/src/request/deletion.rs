mod deletable;
mod kept;
mod verdict;

use std::fmt;
use std::path::{Path, PathBuf};

use meta_schema::Event;

use crate::request::cli_failure::{CliFailure, Invalid};
use crate::request::output::{Listed, Output};
use deletable::Deletable;
use verdict::Verdict;

#[derive(Debug)]
pub struct Deletion {
    paths: Vec<PathBuf>,
    verdicts: Vec<Verdict>,
}

impl Deletion {
    pub fn new(paths: Vec<PathBuf>) -> Self {
        Self { paths, verdicts: vec![] }
    }

    pub fn carry_out(
        self,
        project: &Path,
        mut record: impl FnMut(Event) -> Result<Output, CliFailure>,
    ) -> Result<Output, CliFailure> {
        let verdicts = self
            .paths
            .iter()
            .map(|given| match Deletable::vetted(given, project).and_then(Deletable::delete) {
                Ok(removal) => record(Event::PathRemoved(removal)).map(Verdict::Removed),
                Err(kept) => Ok(Verdict::Refused(given.clone(), kept)),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let done = Self { verdicts, ..self };
        if done.verdicts.iter().any(|verdict| matches!(verdict, Verdict::Refused(..))) {
            Err(CliFailure::raised::<Invalid>(done))
        } else {
            Ok(Output::Listed(Listed::new(done)))
        }
    }
}

impl fmt::Display for Deletion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.verdicts.iter().try_for_each(|verdict| write!(formatter, "{verdict}"))
    }
}
