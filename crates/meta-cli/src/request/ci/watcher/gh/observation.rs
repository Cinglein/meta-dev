use crate::request::ci::ci_status::CiStatus;
use crate::request::output::Listed;

#[derive(Debug)]
pub struct Observation {
    failed_job: Option<Listed>,
    failed_steps: Vec<Listed>,
    finished: Option<CiStatus>,
}

impl Observation {
    pub fn new(failed_steps: Vec<Listed>, failed_job: Option<Listed>, finished: Option<CiStatus>) -> Self {
        Self { failed_job, failed_steps, finished }
    }

    pub fn failed_job(&self) -> Option<&Listed> {
        self.failed_job.as_ref()
    }

    pub fn failed_steps(&self) -> &[Listed] {
        &self.failed_steps
    }

    pub fn finished(&self) -> Option<CiStatus> {
        self.finished
    }
}
