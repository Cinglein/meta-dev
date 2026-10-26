use serde::Deserialize;

use crate::check_name::CheckName;
use crate::verdict::Verdict;

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct RequiredChecks(Vec<CheckName>);

impl RequiredChecks {
    pub fn verdict(&self, seen: impl Fn(&CheckName) -> Option<Verdict>) -> Option<Verdict> {
        self.0.iter().map(|name| seen(name).unwrap_or(Verdict::Pending)).max()
    }
}
