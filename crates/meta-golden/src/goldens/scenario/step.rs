mod call;

use meta_run::Run;
use serde::Deserialize;

use crate::places::Places;
use call::Call;

#[derive(Debug, Deserialize)]
pub struct Step {
    record: Option<String>,
    #[serde(flatten)]
    call: Call,
}

impl Step {
    const PINNED: [(&'static str, &'static str); 5] = [
        ("GIT_CONFIG_GLOBAL", "/dev/null"),
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("GIT_AUTHOR_DATE", "1700000000 +0000"),
        ("GIT_COMMITTER_DATE", "1700000000 +0000"),
        ("LC_ALL", "C"),
    ];

    pub fn record(&self) -> Option<&str> {
        self.record.as_deref()
    }

    pub fn run(&self, places: &Places) -> Run {
        Self::PINNED.iter().fold(self.call.run(places), |run, (name, value)| run.var(name, value))
    }
}
