use std::time::Duration;

use crate::request::ci::ci_status::CiStatus;
use crate::request::output::Listed;

#[derive(Debug)]
pub struct Observation {
    failed_job: Option<Listed>,
    failed_steps: Vec<Listed>,
    status: Option<CiStatus>,
    expected: Option<Duration>,
}

impl Observation {
    pub fn new(
        failed_steps: Vec<Listed>,
        failed_job: Option<Listed>,
        status: Option<CiStatus>,
        expected: Option<Duration>,
    ) -> Self {
        Self { failed_job, failed_steps, status, expected }
    }

    pub fn failed_job(&self) -> Option<&Listed> {
        self.failed_job.as_ref()
    }

    pub fn failed_steps(&self) -> &[Listed] {
        &self.failed_steps
    }

    pub fn finished(&self) -> Option<CiStatus> {
        self.status.and_then(CiStatus::finished)
    }

    pub fn appeared(&self) -> Option<&Self> {
        self.status.map(|_| self)
    }

    pub fn expected(&self) -> Option<Duration> {
        self.expected
    }
}
