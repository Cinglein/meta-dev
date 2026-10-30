use serde::Deserialize;

use super::shell_command::ShellCommand;
use crate::request::dispatch::Prompt;

#[derive(Debug, Default, Deserialize)]
pub struct ToolInput {
    #[serde(default)]
    command: Option<String>,
    #[serde(default)]
    prompt: Option<Prompt>,
}

impl ToolInput {
    pub fn command(&self) -> Option<ShellCommand> {
        self.command.as_deref().map(|text| {
            let Ok(command) = text.parse();
            command
        })
    }

    pub fn prompt(&self) -> Option<&Prompt> {
        self.prompt.as_ref()
    }
}
