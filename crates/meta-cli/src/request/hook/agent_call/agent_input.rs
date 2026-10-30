use serde::Deserialize;

use crate::request::dispatch::Prompt;

#[derive(Debug, Default, Deserialize)]
pub struct AgentInput {
    #[serde(default)]
    prompt: Option<Prompt>,
}

impl AgentInput {
    pub fn prompt(&self) -> Option<&Prompt> {
        self.prompt.as_ref()
    }
}
