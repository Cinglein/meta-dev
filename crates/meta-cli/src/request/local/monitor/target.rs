use std::fmt;

use meta_run::Run;
use serde::{Deserialize, Serialize};

use super::process_row::{GroupId, Pid};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Target {
    Group(GroupId),
    Process(Pid),
}

impl Target {
    pub fn appended(self, run: Run) -> Run {
        run.arg(self.to_string())
    }
}

impl fmt::Display for Target {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Group(group) => write!(formatter, "-{group}"),
            Self::Process(pid) => write!(formatter, "{pid}"),
        }
    }
}
