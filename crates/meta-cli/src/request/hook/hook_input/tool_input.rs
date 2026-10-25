use serde::Deserialize;

use super::shell_command::ShellCommand;

#[derive(Debug, Default, Deserialize)]
pub struct ToolInput {
    #[serde(default)]
    command: Option<ShellCommand>,
}

impl ToolInput {
    pub fn command(&self) -> Option<&ShellCommand> {
        self.command.as_ref()
    }
}
