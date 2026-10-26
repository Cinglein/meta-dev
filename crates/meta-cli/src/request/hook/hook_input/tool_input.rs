use serde::Deserialize;

use super::shell_command::ShellCommand;

#[derive(Debug, Default, Deserialize)]
pub struct ToolInput {
    #[serde(default)]
    command: Option<String>,
}

impl ToolInput {
    pub fn command(&self) -> Option<ShellCommand> {
        self.command.as_deref().map(|text| {
            let Ok(command) = text.parse();
            command
        })
    }
}
