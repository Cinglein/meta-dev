mod check_result;

use serde::Deserialize;

use super::requirement::Requirement;
use crate::live_pull::verdict::Verdict;
use check_result::CheckResult;

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct CheckContext {
    #[serde(default)]
    result: Option<CheckResult>,
    required: Requirement,
}

impl CheckContext {
    pub fn verdict(self) -> Verdict {
        self.result.map_or(Verdict::Pending, CheckResult::verdict)
    }

    pub fn requirement(self) -> Requirement {
        self.required
    }
}
