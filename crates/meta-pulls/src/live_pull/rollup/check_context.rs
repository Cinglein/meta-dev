mod check_result;

use serde::Deserialize;

use crate::check_name::CheckName;
use crate::verdict::Verdict;
use check_result::CheckResult;

#[derive(Clone, Debug, Deserialize)]
pub struct CheckContext {
    #[serde(default)]
    result: Option<CheckResult>,
    name: CheckName,
}

impl CheckContext {
    pub fn verdict(&self) -> Verdict {
        self.result.map_or(Verdict::Pending, CheckResult::verdict)
    }

    pub fn named(&self, name: &CheckName) -> Option<&Self> {
        (self.name == *name).then_some(self)
    }
}
