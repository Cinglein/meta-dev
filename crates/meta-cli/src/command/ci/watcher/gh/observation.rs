use crate::command::ci::ci_status::CiStatus;
use crate::command::output::Listed;

#[derive(Debug)]
pub struct Observation {
    failed_job: Option<Listed>,
    finished: Option<CiStatus>,
}

impl Observation {
    pub fn new(failed_job: Option<Listed>, finished: Option<CiStatus>) -> Self {
        Self { failed_job, finished }
    }

    pub fn failed_job(&self) -> Option<&Listed> {
        self.failed_job.as_ref()
    }

    pub fn finished(&self) -> Option<CiStatus> {
        self.finished
    }
}
