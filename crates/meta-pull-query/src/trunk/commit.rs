use meta_pulls::Page;
use serde::Deserialize;

use super::suite::Suite;

#[derive(Clone, Debug, Deserialize)]
pub struct Commit {
    #[serde(rename = "checkSuites")]
    suites: Page<Suite>,
}

impl Commit {
    pub fn suites(&self) -> impl Iterator<Item = &Suite> {
        self.suites.nodes()
    }
}
