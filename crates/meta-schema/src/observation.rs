use std::fmt;

use meta_acceptance::Verdict;
use serde::{Deserialize, Serialize};

use crate::commit_id::CommitId;
use crate::entity::Project;
use crate::id::Id;
use crate::run_id::RunId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    project: Id<Project>,
    main: CommitId,
    run: RunId,
    verdict: Verdict,
}

impl fmt::Display for Observation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {} at {} run {}", self.project, self.verdict, self.main, self.run)
    }
}
