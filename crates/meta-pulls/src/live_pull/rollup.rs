mod check_context;
mod requirement;

use serde::Deserialize;

use super::checks::Checks;
use super::verdict::Verdict;
use crate::nodes::Nodes;
use check_context::CheckContext;
use requirement::Requirement;

#[derive(Clone, Debug, Deserialize)]
pub struct Rollup {
    contexts: Nodes<CheckContext>,
}

impl Rollup {
    pub fn checks(&self) -> Checks {
        self.worst(Some(Requirement::Required))
            .map(Checks::Required)
            .or_else(|| self.worst(None).map(Checks::Unrequired))
            .unwrap_or(Checks::Absent)
    }

    fn worst(&self, among: Option<Requirement>) -> Option<Verdict> {
        let counted = |context: &CheckContext| among.is_none_or(|requirement| context.requirement() == requirement);
        self.contexts.iter().copied().filter(counted).map(CheckContext::verdict).max()
    }
}
