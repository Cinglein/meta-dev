use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum ToolName {
    Agent,
    #[serde(other)]
    Other,
}
