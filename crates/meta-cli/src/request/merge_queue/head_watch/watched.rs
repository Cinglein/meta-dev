use meta_pulls::{GhRefusal, Stop};
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
    pub fn noted(self, refusal: &GhRefusal) -> Self {
        let note = Listed::new(format_args!("gh reported a failure, yet the PR merged: {refusal}\n"));
        match self {
            Self::Left(shown) => Self::Left(note.then(shown)),
            Self::Flagged(shown, pull_request) => Self::Flagged(note.then(shown), pull_request),
            stopped @ Self::Stopped(_) => stopped,
        }
    }

    pub fn folded(self, ran: QueueRun) -> QueueRun {
        match self {
            Self::Left(shown) => ran.landed(shown),
            Self::Flagged(shown, pull_request) => ran.flagged(shown, pull_request),
            Self::Stopped(stop) => ran.halted(&stop),
        }
    }
}
