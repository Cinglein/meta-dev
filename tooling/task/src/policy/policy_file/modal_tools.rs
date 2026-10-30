mod leased_tools;

use meta_schema::ToolName;
use serde::{Deserialize, Serialize};

use leased_tools::LeasedTools;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModalTools {
    modal: Vec<ToolName>,
    leased: LeasedTools,
}

#[cfg(test)]
mod tests {
    use super::ModalTools;

    const UNGATED: [&str; 2] = ["modal = []\nleased = []\n", "modal = []\n"];

    #[test]
    fn tools_without_a_leased_tool_do_not_parse() {
        assert!(
            UNGATED
                .iter()
                .all(|text| toml::from_str::<ModalTools>(text).is_err())
        );
    }
}
