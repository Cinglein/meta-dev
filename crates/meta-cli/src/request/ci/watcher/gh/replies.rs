use meta_golden::{Goldens, Recording};
use meta_run::GhArgv;

use super::{Gh, Observation};

pub struct Replies;

impl Replies {
    const SCENARIO: &'static str = "gh-commit-checks";

    pub fn recorded(call: GhArgv) -> Vec<Recording> {
        let recordings = Recording::all_in(&Goldens::workspace().dir().join(Self::SCENARIO)).unwrap();
        recordings.into_iter().filter(|recording| recording.argv().calls(call)).collect()
    }

    pub fn about(sha: &str) -> Recording {
        Self::recorded(GhArgv::CommitChecks).into_iter().find(|reply| reply.argv().to_string().contains(sha)).unwrap()
    }

    pub fn observed(sha: &str) -> Observation {
        Observation::from(&Gh::checks_in(Self::about(sha).out()).unwrap())
    }
}
