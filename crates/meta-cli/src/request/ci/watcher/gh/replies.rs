use meta_golden::{Goldens, Recording};
use meta_run::GhArgv;

use super::{Gh, Observation};
use crate::request::ci::ci_status::CiStatus;

pub struct Replies;

impl Replies {
    pub fn recorded(call: GhArgv) -> Vec<Recording> {
        let recordings = Goldens::workspace().recordings().unwrap();
        recordings.into_iter().filter(|recording| recording.argv().calls(call)).collect()
    }

    pub fn about(sha: &str) -> Recording {
        Self::recorded(GhArgv::CommitChecks).into_iter().find(|reply| reply.argv().to_string().contains(sha)).unwrap()
    }

    pub fn observed(sha: &str) -> Observation {
        Observation::from(&Gh::checks_in(Self::about(sha).out()).unwrap())
    }

    pub fn reporting(status: CiStatus) -> Observation {
        Observation::new(Vec::new(), None, Some(status), None)
    }
}
