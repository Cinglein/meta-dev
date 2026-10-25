mod shell_command;
mod tool_input;

use std::path::PathBuf;

use meta_schema::Actor;
use serde::Deserialize;

use crate::command::ci::SessionId;

use shell_command::ShellCommand;
use tool_input::ToolInput;

#[derive(Debug, Default, Deserialize)]
pub struct HookInput {
    #[serde(default)]
    session_id: Option<SessionId>,
    #[serde(default)]
    cwd: Option<PathBuf>,
    #[serde(default)]
    tool_input: Option<ToolInput>,
}

impl HookInput {
    pub fn cwd(&self) -> PathBuf {
        self.cwd.clone().or_else(|| std::env::current_dir().ok()).unwrap_or_default()
    }

    pub fn command(&self) -> Option<&ShellCommand> {
        self.tool_input.as_ref().and_then(ToolInput::command)
    }

    pub fn session(&self) -> Option<&SessionId> {
        self.session_id.as_ref()
    }

    pub fn announced_actor(&self) -> Option<Actor> {
        let configured = std::env::var("META_ACTOR").ok().and_then(|name| name.parse().ok());
        configured.or_else(|| self.session_id.as_ref()?.announce())
    }
}
