mod identity;
mod pushed_commits;
mod shell_command;
mod tool_input;

use std::path::PathBuf;
use std::time::Instant;

use chrono::{DateTime, Utc};
use meta_schema::Actor;
use serde::Deserialize;

use super::CallPhase;
use crate::request::ci::{SessionDir, SessionId, SessionStart};
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

    pub fn refusal(&self, deadline: Instant) -> Option<Listed> {
        let command = self.command()?;
        let directories = command.commit_directories(&self.cwd());
        let foreign = directories.iter().find_map(|directory| Identity::refusal_in(directory, deadline));
        let writing = self.session().zip(command.writes());
        let unrecorded = writing.and_then(|(session, ())| SessionStart::for_session(session).record().err());
        unrecorded.or(foreign)
    }

    pub fn unpushable(&self, project: &Project, subcommand: &str, deadline: Instant) -> Option<Listed> {
        self.command()?.runs(subcommand)?;
        PushedCommits::refusal_in(project.root(), project.config().remote().ok()?, deadline)
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

    use super::{CallPhase, HookInput};
    use crate::request::ci::SessionDir;

    const TASK: &str = "task-00000000000000c0";
    const GIT: &str = ".git";

    #[test]
    fn only_a_session_working_in_a_task_worktree_stamps_that_task_whatever_paths_others_touch() {
        let root = tempfile::tempdir().unwrap();
        let worktree = root.path().join(TASK);
        std::fs::create_dir_all(worktree.join(GIT)).unwrap();
        let stamp = |input: serde_json::Value| {
            let input = serde_json::from_value::<HookInput>(input).unwrap();
            input.stamp(CallPhase::Finished, Utc::now()).map(|stamped| stamped.is_ok())
        };
        let edit = json!({ "cwd": root.path(), "tool_input": { "file_path": worktree.join("a.rs") } });
        let shell = json!({ "cwd": root.path(), "tool_input": { "command": format!("cd {TASK} && cargo build") } });
        let own = json!({ "cwd": worktree, "tool_input": { "command": "ls" } });
        assert_eq!([edit, shell, own].map(stamp), [None, None, Some(true)]);
        SessionDir::stamped(&worktree, CallPhase::Started, DateTime::<Utc>::MIN_UTC).unwrap().unwrap();
        let read = SessionDir::stamp_of(&worktree.join(GIT).canonicalize().unwrap(), TASK.parse().unwrap()).unwrap();
        let kept = read.map(|stamp| (stamp.worktree().task().to_string(), stamp.call()));
        assert_eq!(kept, Some((TASK.to_owned(), CallPhase::Finished)));
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
