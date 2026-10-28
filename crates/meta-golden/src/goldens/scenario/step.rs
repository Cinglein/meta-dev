mod call;
mod setup;

use meta_run::{EnvPolicy, Run};
use serde::Deserialize;

use crate::goldens::Reach;
use crate::places::Places;
use call::Call;
use setup::Setup;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ScenarioStep {
    Recorded {
        record: String,
        #[serde(flatten)]
        call: Call,
    },
    Setup(Setup),
}

impl ScenarioStep {
    const PINNED: [(&'static str, &'static str); 9] = [
        ("GIT_CONFIG_GLOBAL", "/dev/null"),
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("GIT_CONFIG_COUNT", "1"),
        ("GIT_CONFIG_KEY_0", "init.defaultBranch"),
        ("GIT_CONFIG_VALUE_0", "main"),
        ("GIT_AUTHOR_DATE", "1700000000 +0000"),
        ("GIT_COMMITTER_DATE", "1700000000 +0000"),
        ("GIT_TERMINAL_PROMPT", "0"),
        ("LC_ALL", "C"),
    ];
    const CEILING: &'static str = "GIT_CEILING_DIRECTORIES";

    pub fn reach(&self) -> Reach {
        match self {
            Self::Recorded { call, .. } => call.reach(),
            Self::Setup(_) => Reach::Offline,
        }
    }

    pub fn record(&self) -> Option<&str> {
        match self {
            Self::Recorded { record, .. } => Some(record),
            Self::Setup(_) => None,
        }
    }

    pub fn run(&self, places: &Places) -> Run {
        let run = match self {
            Self::Recorded { call, .. } => call.run(places),
            Self::Setup(setup) => setup.run(places),
        };
        let ceiling = places.root().parent().unwrap_or(places.root());
        let isolated = run.policed(EnvPolicy::NoGitVars).var(Self::CEILING, ceiling);
        Self::PINNED.iter().fold(isolated, |run, (name, value)| run.var(name, value))
    }
}
