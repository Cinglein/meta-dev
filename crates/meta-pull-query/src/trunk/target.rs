use meta_pulls::Page;
use serde::Deserialize;

use super::commit::Commit;
use super::suite::Suite;

#[derive(Clone, Debug, Deserialize)]
pub struct Target {
    history: Page<Commit>,
}

impl Target {
    pub fn suites(&self) -> impl Iterator<Item = &Suite> {
        self.history.nodes().flat_map(Commit::suites)
    }
}
