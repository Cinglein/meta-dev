mod leased_tools;

use meta_schema::ToolName;
use serde::Deserialize;

use crate::request::output::Listed;
use leased_tools::LeasedTools;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolPolicy {
    #[serde(default)]
    modal: Vec<ToolName>,
    leased: LeasedTools,
}

impl ToolPolicy {
    const MODAL_REFUSED: &'static str = "Modal questions are refused: write the decision as discussion in your reply (problem, options with tradeoffs, recommendation) and keep working on what doesn't depend on it.";

    pub fn refusal(&self, tool: Option<&ToolName>) -> Option<Listed> {
        tool.filter(|tool| self.modal.contains(tool)).map(|_| Listed::new(Self::MODAL_REFUSED))
    }

    pub fn leased(&self, tool: Option<&ToolName>) -> Option<()> {
        tool.and_then(|tool| self.leased.gates(tool))
    }
}

#[cfg(test)]
mod tests {
    use super::ToolPolicy;

    const UNGATED: [&str; 2] = ["modal = []\nleased = []\n", "modal = []\n"];

    #[test]
    fn a_tool_policy_without_a_leased_tool_does_not_parse() {
        assert!(UNGATED.iter().all(|text| toml::from_str::<ToolPolicy>(text).is_err()));
    }
}
