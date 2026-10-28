use meta_gh::GhRun;
use serde::Deserialize;

use super::gh_jobs::GhJobs;

#[derive(Clone, Debug, Deserialize)]
pub struct GhSuite {
    #[serde(flatten)]
    run: GhRun,
    #[serde(default, rename = "checkRuns")]
    jobs: GhJobs,
    #[serde(default, rename = "otherJobs")]
    others: GhJobs,
}

impl GhSuite {
    pub fn of_actions(&self) -> Option<&Self> {
        self.run.database_id().map(|_| self)
    }

    pub fn run(&self) -> &GhRun {
        &self.run
    }

    pub fn jobs(&self) -> &GhJobs {
        &self.jobs
    }

    pub fn unstarted(&self) -> Option<&GhJobs> {
        self.jobs.unstarted().filter(|_| self.others.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::GhSuite;

    const SUITE: &str = r#"{"status":"COMPLETED","conclusion":"FAILURE","workflowRun":{},"checkRuns":{"nodes":[{"name":"a","conclusion":"FAILURE","steps":{"nodes":[]}}]}"#;
    const OTHER: &str = r#","otherJobs":{"nodes":[{"name":"b","conclusion":"SUCCESS"}]}}"#;
    const NO_OTHER: &str = r#","otherJobs":{"nodes":[]}}"#;
    const UNASKED: &str = "}";

    #[test]
    fn a_suite_never_started_only_when_no_job_besides_the_failed_ones_exists() {
        let [alone, beside, unasked] =
            [NO_OTHER, OTHER, UNASKED].map(|rest| serde_json::from_str::<GhSuite>(&[SUITE, rest].concat()).unwrap());
        assert!(alone.unstarted().is_some() && beside.unstarted().is_none() && unasked.unstarted().is_some());
    }
}
