mod cargo_tag;
mod deletable;
mod kept;
mod occupancy;
mod verdict;
mod vetted;
mod work_tree;

use std::fmt;
use std::path::{Path, PathBuf};

use meta_schema::Event;

use crate::request::cli_failure::{CliFailure, Invalid};
use crate::request::output::{Listed, Output};
use deletable::Deletable;
use kept::Kept;
use occupancy::Occupancy;
use verdict::Verdict;
use vetted::Vetted;

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
        let vetted: Vec<Result<Vetted, Kept>> =
            self.paths.iter().map(|given| Vetted::checked(given, project)).collect();
        let occupancy = Occupancy::scanned(&vetted.iter().flatten().map(Vetted::path).collect::<Vec<_>>());
        let verdicts = self
            .paths
            .iter()
            .zip(vetted)
            .map(|(given, vetted)| {
                match vetted.and_then(|vetted| Deletable::vacated(&vetted, &occupancy)).and_then(Deletable::delete) {
                    Ok(removal) => record(Event::PathRemoved(removal)).map(Verdict::Removed),
                    Err(kept) => Ok(Verdict::Refused(given.clone(), kept)),
                }
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
