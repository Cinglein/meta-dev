use meta_schema::ToolName;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Vec<ToolName>", into = "Vec<ToolName>")]
pub struct LeasedTools(Vec<ToolName>);

impl LeasedTools {
    const EMPTY: &'static str = "[tools] leased must name at least one tool";
}

impl TryFrom<Vec<ToolName>> for LeasedTools {
    type Error = &'static str;

    fn try_from(tools: Vec<ToolName>) -> Result<Self, Self::Error> {
        tools
            .first()
            .map(|_| Self(tools.clone()))
            .ok_or(Self::EMPTY)
    }
}

impl From<LeasedTools> for Vec<ToolName> {
    fn from(tools: LeasedTools) -> Self {
        tools.0
    }
}
