use meta_schema::ToolName;
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "Vec<ToolName>")]
pub struct LeasedTools(Vec<ToolName>);

impl LeasedTools {
    const EMPTY: &'static str = "[tools] leased must name at least one tool";

    pub fn gates(&self, tool: &ToolName) -> Option<()> {
        self.0.contains(tool).then_some(())
    }
}

impl TryFrom<Vec<ToolName>> for LeasedTools {
    type Error = &'static str;

    fn try_from(tools: Vec<ToolName>) -> Result<Self, Self::Error> {
        tools.first().map(|_| Self(tools.clone())).ok_or(Self::EMPTY)
    }
}
