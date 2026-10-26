mod cargo_target;
mod footprint;
mod intent;
mod kept;
mod occupancy;
mod verdict;
mod vetted;
mod work_tree;

use std::cell::LazyCell;
use std::fmt;
use std::path::{Path, PathBuf};

use meta_schema::Event;

use crate::request::cli_failure::{CliFailure, Invalid};
use crate::request::output::{Listed, Output};
use intent::Intent;
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
        let intended = self
            .paths
            .iter()
            .map(|given| match Vetted::checked(given, project) {
                Ok(vetted) => Intent::announced(vetted, &mut record).map(Ok),
                Err(kept) => Ok(Err(kept)),
            })
            .collect::<Result<Vec<Result<Intent, Kept>>, CliFailure>>()?;
        let candidates: Vec<PathBuf> =
            intended.iter().flatten().map(|intent| intent.vetted().path().to_path_buf()).collect();
        let occupancy = LazyCell::new(|| Occupancy::scanned(&candidates));
        let verdicts = self
            .paths
            .iter()
            .zip(intended)
            .map(|(given, intent)| match intent.and_then(|intent| intent.carried_out(&occupancy, project)) {
                Ok(logged) => Verdict::Removed(logged),
                Err(kept) => Verdict::Refused(given.clone(), kept),
            })
            .collect();
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
