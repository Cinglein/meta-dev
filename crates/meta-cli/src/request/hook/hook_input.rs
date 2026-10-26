mod identity;
mod shell_command;
mod tool_input;

use std::path::PathBuf;

use meta_schema::Actor;
use serde::Deserialize;

use crate::request::ci::{SessionId, SessionStart};
use crate::request::output::Listed;

use identity::Identity;
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
    const PR_CREATE: &'static [&'static str] = &["gh", "pr", "create"];

    pub fn cwd(&self) -> PathBuf {
        self.cwd.clone().or_else(|| std::env::current_dir().ok()).unwrap_or_default()
    }

    pub fn command(&self) -> Option<ShellCommand> {
        self.tool_input.as_ref().and_then(ToolInput::command)
    }

    pub fn open_session(&self) -> Result<(), Listed> {
        self.session().map_or(Ok(()), |session| SessionStart::for_session(session).open().map_err(Listed::new))
    }

    pub fn refusal(&self) -> Option<Listed> {
        let command = self.command()?;
        let foreign =
            command.commit_directories(&self.cwd()).iter().find_map(|directory| Identity::refusal_in(directory));
        let writing = self.session().zip(command.writes());
        let unrecorded = writing.and_then(|(session, ())| SessionStart::for_session(session).record().err());
        unrecorded.or(foreign)
    }

    pub fn publishing(&self) -> Option<()> {
        let command = self.command()?;
        command.writes().or_else(|| command.invokes(Self::PR_CREATE))
    }

    pub fn session(&self) -> Option<&SessionId> {
        self.session_id.as_ref()
    }

    pub fn announced_actor(&self) -> Option<Actor> {
        let configured = std::env::var("META_ACTOR").ok().and_then(|name| name.parse().ok());
        self.session_id.as_ref().map_or(configured.clone(), |session| session.announce(configured.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::HookInput;

    const PUBLISHING: [&str; 3] = ["git push -u origin x", "cd a && gh pr create --fill", "git commit -m y"];
    const QUIET: [&str; 3] = ["gh pr list", "git status", "gh pr view 1"];

    #[test]
    fn pushes_commits_and_pull_request_creation_publish_and_nothing_else() {
        let publishing = |command: &str| {
            let input = serde_json::json!({ "tool_input": { "command": command } }).to_string();
            serde_json::from_str::<HookInput>(&input).unwrap().publishing().is_some()
        };
        assert!(PUBLISHING.iter().all(|command| publishing(command)));
        assert!(QUIET.iter().all(|command| !publishing(command)));
    }
}
