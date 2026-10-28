mod ancestor;

use std::collections::BinaryHeap;
use std::ffi::OsString;
use std::time::Duration;

use meta_gh::Nodes;
use serde::Deserialize;

use super::gh_jobs::GhJobs;
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

    pub fn jobs(&self) -> GhJobs {
        GhJobs::joined(self.actions().into_iter().map(GhSuite::jobs))
    }

    pub fn failed_job_ids(&self) -> Vec<OsString> {
        self.actions().iter().flat_map(|suite| suite.jobs().failed_job_ids()).collect()
    }

    pub fn expected(&self) -> Option<Duration> {
        let took = self.history.nodes().iter().filter_map(Ancestor::took).collect::<BinaryHeap<_>>().into_sorted_vec();
        took.get(took.len() >> 1).copied()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::CommitChecks;

    const CHECKS: &str = r#"{"checkSuites":{"nodes":[{"status":"QUEUED"},{"status":"IN_PROGRESS","workflowRun":{}}]},
        "history":{"nodes":[{"checkSuites":{"nodes":[{"status":"IN_PROGRESS","workflowRun":{}}]}},
        {"checkSuites":{"nodes":[{"status":"COMPLETED","workflowRun":{},"createdAt":"2026-10-27T10:00:00Z","updatedAt":"2026-10-27T10:00:09Z"}]}},
        {"checkSuites":{"nodes":[{"status":"COMPLETED","workflowRun":{},"createdAt":"2026-10-27T10:00:00Z","updatedAt":"2026-10-27T10:00:01Z"},
            {"status":"COMPLETED","workflowRun":{},"createdAt":"2026-10-27T10:00:00Z","updatedAt":"2026-10-27T10:00:04Z"}]}},
        {"checkSuites":{"nodes":[{"status":"COMPLETED","workflowRun":{},"createdAt":"2026-10-27T10:00:00Z","updatedAt":"2026-10-27T10:00:02Z"}]}},
        {"checkSuites":{"nodes":[{"status":"COMPLETED","workflowRun":{},"createdAt":"2026-10-27T10:00:00Z","updatedAt":"2026-10-27T10:00:06Z"}]}},
        {"checkSuites":{"nodes":[{"status":"COMPLETED","workflowRun":{},"createdAt":"2026-10-27T10:00:00Z","updatedAt":"2026-10-27T10:00:01Z"}]}}]}}"#;
    const MEDIAN: Duration = Duration::from_secs(4);

    #[test]
    fn only_actions_runs_count_and_the_expected_run_is_the_median_of_finished_ancestors() {
        let checks: CommitChecks = serde_json::from_str(CHECKS).unwrap();
        assert!(checks.actions().len() == 1 && checks.expected() == Some(MEDIAN));
    }
}
