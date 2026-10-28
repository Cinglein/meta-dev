use std::time::Duration;

use meta_gh::{GhRun, Nodes};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Ancestor {
    #[serde(default, rename = "checkSuites")]
    suites: Nodes<GhRun>,
}

impl Ancestor {
    pub fn took(&self) -> Option<Duration> {
        let runs = self.suites.nodes().iter().filter(|run| run.database_id().is_some());
        let took: Vec<Duration> = runs.map(GhRun::took).collect::<Option<_>>()?;
        took.into_iter().max()
    }
}
