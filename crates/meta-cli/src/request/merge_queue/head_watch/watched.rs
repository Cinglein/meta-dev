use meta_pulls::Stop;
use meta_schema::PullRequest;

use crate::request::merge_queue::queue_run::QueueRun;
use crate::request::output::Listed;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Watched {
    Left(Listed),
    Flagged(Listed, PullRequest),
    Stopped(Stop),
}

impl Watched {
    pub fn folded(self, ran: QueueRun) -> QueueRun {
        match self {
            Self::Left(shown) => ran.landed(shown),
            Self::Flagged(shown, pull_request) => ran.flagged(shown, pull_request),
            Self::Stopped(stop) => ran.halted(&stop),
        }
    }
}
