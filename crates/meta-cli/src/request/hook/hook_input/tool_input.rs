use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::shell_command::ShellCommand;

#[derive(Debug, Default, Deserialize)]
pub struct ToolInput {
    #[serde(default)]
    command: Option<String>,
    #[serde(default)]
    file_path: Option<PathBuf>,
}

impl ToolInput {
    pub fn command(&self) -> Option<ShellCommand> {
        self.command.as_deref().map(|text| {
            let Ok(command) = text.parse();
            command
        })
    }

    pub fn file_path(&self) -> Option<&Path> {
        self.file_path.as_deref()
    }
}
