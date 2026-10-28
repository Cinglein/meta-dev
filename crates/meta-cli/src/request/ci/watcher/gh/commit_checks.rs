mod ancestor;

use std::collections::BinaryHeap;
use std::ffi::OsString;
use std::time::Duration;

use meta_gh::Nodes;
use serde::Deserialize;

use super::gh_suite::GhSuite;
use ancestor::Ancestor;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct CommitChecks {
    #[serde(default, rename = "checkSuites")]
    suites: Nodes<GhSuite>,
    #[serde(default)]
    history: Nodes<Ancestor>,
}

impl CommitChecks {
    pub fn actions(&self) -> Vec<&GhSuite> {
        self.suites.nodes().iter().filter_map(GhSuite::of_actions).collect()
    }

    pub fn failed_job_ids(&self) -> Vec<OsString> {
        self.actions().iter().flat_map(|suite| suite.jobs().failed_job_ids()).collect()
    }

    pub fn expected(&self) -> Option<Duration> {
        let took = self.history.nodes().iter().filter_map(Ancestor::took).collect::<BinaryHeap<_>>().into_sorted_vec();
        took.get(took.len() >> 1).copied()
    }
}
