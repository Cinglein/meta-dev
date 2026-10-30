mod agent_call;
mod identity;
mod pushed_commits;
mod shell_command;
mod tool_input;
mod unreadable;

use std::io::Read as _;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use meta_schema::{Actor, ToolName};
use serde::Deserialize;
use serde::de::IgnoredAny;

use super::CallPhase;
use crate::request::ci::{SessionDir, SessionId, SessionStart};
use crate::request::output::Listed;
use crate::request::project::Project;
use crate::request::scale_policy::ToolPolicy;

use agent_call::AgentCall;
use identity::Identity;
use pushed_commits::PushedCommits;
use shell_command::ShellCommand;
use tool_input::ToolInput;
pub use unreadable::Unreadable;

#[derive(Debug, Default, Deserialize)]
pub struct HookInput {
    #[serde(default)]
    session_id: Option<SessionId>,
    #[serde(default)]
    cwd: Option<PathBuf>,
    #[serde(default)]
    tool_input: Option<ToolInput>,
    #[serde(default)]
    tool_name: Option<ToolName>,
    #[serde(default)]
    agent_id: Option<IgnoredAny>,
}

impl HookInput {
    const UNNAMED: &'static str = "the hook input names no tool, so no tool call is allowed";

    pub fn from_stdin() -> Result<HookInput, Unreadable> {
        let mut bytes = Vec::new();
        std::io::stdin().read_to_end(&mut bytes)?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    pub fn cwd(&self) -> PathBuf {
        self.cwd.clone().or_else(|| std::env::current_dir().ok()).unwrap_or_default()
    }

    pub fn command(&self) -> Option<ShellCommand> {
        self.tool_input.as_ref().and_then(ToolInput::command)
    }

    pub fn open_session(&self) -> Result<(), Listed> {
        self.session().map_or(Ok(()), |session| SessionStart::for_session(session).open().map_err(Listed::new))
    }

    pub fn modal(&self, policy: &ToolPolicy) -> Option<Listed> {
        policy.refusal(self.tool_name.as_ref())
    }

    pub fn tool(&self) -> Result<&ToolName, Listed> {
        self.tool_name.as_ref().ok_or_else(|| Listed::new(Self::UNNAMED))
    }

    pub fn agent_call(&self, policy: &ToolPolicy) -> Option<AgentCall<'_>> {
        policy.leased(self.tool_name.as_ref())?;
        let prompt = self.tool_input.as_ref().and_then(ToolInput::prompt);
        Some(AgentCall::new(self.cwd(), prompt, self.agent_id.as_ref().map(|_| ())))
    }

    pub fn refusal(&self) -> Option<Listed> {
        let command = self.command()?;
        let directories = command.commit_directories(&self.cwd());
        let foreign = directories.iter().find_map(|directory| Identity::refusal_in(directory));
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

    pub fn stamp(&self, call: CallPhase, at: DateTime<Utc>) -> Option<Result<(), Listed>> {
        SessionDir::stamped(&self.cwd(), call, at)
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
    use chrono::{DateTime, Utc};
    use serde_json::json;

    use super::super::HookEvent;
    use super::{CallPhase, HookInput};
    use crate::request::ci::SessionDir;

    const TASK: &str = "task-00000000000000c0";
    const GIT: &str = ".git";
    const FILE: &str = "a.rs";
    const SHELL: &str = "cd task-00000000000000c0 && cargo build";
    const LISTING: &str = "ls";
    const STAMPED: [Option<bool>; 3] = [None, None, Some(true)];
    const FINISHED: CallPhase = CallPhase::Finished;
    const STARTED: CallPhase = CallPhase::Started;
    const KEPT: [Option<CallPhase>; 2] = [Some(CallPhase::Finished), Some(CallPhase::Started)];
    const EVENTS: [HookEvent; 3] = [HookEvent::PreToolUse, HookEvent::PostToolUse, HookEvent::Stop];
    const PHASES: [Option<CallPhase>; 3] = [Some(CallPhase::Started), Some(CallPhase::Finished), None];

    #[test]
    fn only_a_session_working_in_a_task_worktree_stamps_that_task_and_an_older_stamp_never_wins() {
        let root = tempfile::tempdir().unwrap();
        let worktree = root.path().join(TASK);
        std::fs::create_dir_all(worktree.join(GIT)).unwrap();
        let at = Utc::now();
        let stamp = |input: serde_json::Value| {
            let input = serde_json::from_value::<HookInput>(input).unwrap();
            input.stamp(FINISHED, at).map(|stamped| stamped.is_ok())
        };
        let edit = json!({ "cwd": root.path(), "tool_input": { "file_path": worktree.join(FILE) } });
        let shell = json!({ "cwd": root.path(), "tool_input": { "command": SHELL } });
        let own = json!({ "cwd": worktree, "tool_input": { "command": LISTING } });
        assert_eq!([edit, shell, own].map(stamp), STAMPED);
        let kept = [DateTime::<Utc>::MIN_UTC, at].map(|when| {
            SessionDir::stamped(&worktree, STARTED, when).unwrap().unwrap();
            let read = SessionDir::stamp_of(&worktree.join(GIT).canonicalize().unwrap(), TASK.parse().unwrap());
            read.unwrap().map(|stamp| stamp.call())
        });
        assert_eq!(kept, KEPT);
    }

    #[test]
    fn only_tool_use_hooks_stamp_a_call_phase() {
        assert_eq!(EVENTS.map(HookEvent::phase), PHASES);
    }

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
