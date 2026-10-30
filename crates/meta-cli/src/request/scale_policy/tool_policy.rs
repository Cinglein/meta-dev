use meta_schema::ToolName;
use serde::Deserialize;

use crate::request::output::Listed;

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolPolicy {
    #[serde(default)]
    modal: Vec<ToolName>,
}

impl ToolPolicy {
    const MODAL_REFUSED: &'static str = "Modal questions are refused: write the decision as discussion in your reply (problem, options with tradeoffs, recommendation) and keep working on what doesn't depend on it.";

    pub fn refusal(&self, tool: Option<&ToolName>) -> Option<Listed> {
        tool.filter(|tool| self.modal.contains(tool)).map(|_| Listed::new(Self::MODAL_REFUSED))
    }
}
