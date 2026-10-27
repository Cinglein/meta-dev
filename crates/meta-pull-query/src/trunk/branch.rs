use serde::Deserialize;

use super::suite::Suite;
use super::target::Target;

#[derive(Clone, Debug, Deserialize)]
pub struct Branch {
    target: Target,
}

impl Branch {
    pub fn suites(&self) -> impl Iterator<Item = &Suite> {
        self.target.suites()
    }
}
