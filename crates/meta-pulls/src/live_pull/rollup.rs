mod check_context;

use serde::Deserialize;

use super::checks::Checks;
use crate::check_name::CheckName;
use crate::failing_checks::FailingChecks;
use crate::page::Page;
use crate::required_checks::RequiredChecks;
use check_context::CheckContext;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Rollup {
    contexts: Page<CheckContext>,
}

impl Rollup {
    pub fn silent(&self) -> bool {
        self.contexts.nodes().next().is_none()
    }

    pub fn failing(&self) -> FailingChecks {
        self.contexts.nodes().filter_map(CheckContext::failing_name).collect()
    }

    pub fn checks(&self, required: &RequiredChecks) -> Checks {
        let named = |name: &CheckName| {
            self.contexts.nodes().filter_map(|context| context.named(name)).map(CheckContext::verdict).max()
        };
        let any = self.contexts.nodes().map(CheckContext::verdict).max();
        self.contexts
            .page_info()
            .truncated()
            .map(|()| Checks::Truncated)
            .or_else(|| required.verdict(named).map(Checks::Required))
            .or_else(|| any.map(Checks::Unrequired))
            .unwrap_or(Checks::Absent)
    }
}
