mod identity;
mod pushed_commits;
mod shell_command;
mod tool_input;

use std::path::PathBuf;

use meta_schema::Actor;
use serde::Deserialize;

use crate::request::ci::{SessionId, SessionStart};
use crate::request::output::Listed;
use crate::request::project::Project;

use identity::Identity;
use pushed_commits::PushedCommits;
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

    pub fn unpushable(&self, project: &Project, subcommand: &str) -> Option<Listed> {
        self.command()?.runs(subcommand)?;
        PushedCommits::refusal_in(project.root(), project.config().remote().ok()?)
    }

    pub fn publishing(&self) -> Option<PathBuf> {
        self.command()?.publishing_directory(&self.cwd())
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

    const PUBLISHING: [(&str, &str); 4] = [
        ("git push -u origin x", "/w"),
        ("cd a && gh pr create --fill", "/w/a"),
        ("git status; git -C b commit -m y", "/w/b"),
        ("(cd c && git -C d push) && git push", "/w/c/d"),
    ];
    const QUIET: [&str; 3] = ["gh pr list", "git status", "cd a && gh pr view 1"];

    #[test]
    fn pushes_commits_and_pull_request_creation_publish_from_their_own_directory_and_nothing_else() {
        let publishing = |command: &str| {
            let input = serde_json::json!({ "cwd": "/w", "tool_input": { "command": command } }).to_string();
            serde_json::from_str::<HookInput>(&input).unwrap().publishing()
        };
        assert!(PUBLISHING.iter().all(|(command, directory)| publishing(command) == Some((*directory).into())));
        assert!(QUIET.iter().all(|command| publishing(command).is_none()));
    }
}
