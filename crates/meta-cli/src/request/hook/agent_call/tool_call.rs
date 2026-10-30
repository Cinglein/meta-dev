mod tool_name;

use serde::Deserialize;

use tool_name::ToolName;

#[derive(Debug, Default, Deserialize)]
pub struct ToolCall {
    #[serde(default)]
    tool_name: Option<ToolName>,
}

impl ToolCall {
    pub fn agent(stdin: &str) -> Option<()> {
        let call = serde_json::from_str::<Self>(stdin).unwrap_or_default();
        call.tool_name.filter(|name| *name == ToolName::Agent).map(|_| ())
    }
}
