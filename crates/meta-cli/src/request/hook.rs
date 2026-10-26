mod hook_input;
mod reply;

use std::fmt;
use std::str::FromStr;

use meta_listing::Scope;
use meta_schema::Actor;

use crate::request::ci::{SessionDir, Watcher};
use crate::request::cli_failure::{CliFailure, Invalid, Misuse};
use crate::request::output::{Listed, Output};
use crate::request::project::Project;
use hook_input::HookInput;
use reply::{Blocking, Denial, Reply, Verdict};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HookEvent {
    SessionStart,
    PreToolUse,
    PostToolUse,
    Stop,
}

impl FromStr for HookEvent {
    type Err = CliFailure;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "session-start" => Ok(Self::SessionStart),
            "pre-tool-use" => Ok(Self::PreToolUse),
            "post-tool-use" => Ok(Self::PostToolUse),
            "stop" => Ok(Self::Stop),
            other => Err(CliFailure::raised::<Misuse>(format!("unknown hook `{other}`"))),
        }
    }
}

impl fmt::Display for HookEvent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::SessionStart => "SessionStart",
            Self::PreToolUse => "PreToolUse",
            Self::PostToolUse => "PostToolUse",
            Self::Stop => "Stop",
        })
    }
}

impl HookEvent {
    const COMMIT: &'static str = "commit";

    pub fn respond(self) -> Result<Output, CliFailure> {
        let stdin = std::io::read_to_string(std::io::stdin()).map_err(CliFailure::raised::<Invalid>)?;
        let input: HookInput = serde_json::from_str(&stdin).map_err(CliFailure::raised::<Invalid>)?;
        let reply = Project::try_from(input.cwd().as_path()).ok().map(|project| {
            let session = input.session().and_then(|session| SessionDir::for_session(&project, session));
            self.reply(&project, &input).with_news(session.and_then(|dir| dir.news()))
        });
        let opened = (self == Self::SessionStart).then(|| input.open_session()).and_then(Result::err);
        let reply = reply.or_else(|| self.outside_projects(&input)).map(|reply| reply.with_news(opened));
        Ok(Output::Listed(Listed::new(reply.map(|reply| reply.to_string()).unwrap_or_default())))
    }

    fn reply(self, project: &Project, input: &HookInput) -> Reply {
        match self {
            Self::SessionStart => {
                let actor = input.announced_actor().unwrap_or_else(Actor::owner);
                let state = project
                    .log()
                    .and_then(|log| Listed::from_log(&log, None, Scope::Open))
                    .map_or_else(|failure| format!("meta/log unreadable: {failure}\n"), |listed| listed.to_string());
                let shown = format!(
                    "state of meta/log. actor: {actor}. meta list | claim | done | release | issue | concern | open (issue, concern and open need --division <path>)\n{state}"
                );
                Reply::new(self, Verdict::Allow, Some(Listed::new(shown)))
            }
            Self::PreToolUse => Reply::new(
                self,
                input
                    .command()
                    .and_then(|command| command.refusal(project.config()))
                    .or_else(|| input.refusal())
                    .map(Denial::from)
                    .map_or(Verdict::Allow, Verdict::Deny),
                None,
            ),
            Self::PostToolUse => Reply::new(self, Verdict::Allow, Self::after_push(project, input)),
            Self::Stop => Reply::new(
                self,
                input
                    .session()
                    .and_then(|session| Watcher::stop_reason(project, session))
                    .map(Blocking::from)
                    .map_or(Verdict::Allow, Verdict::Block),
                None,
            ),
        }
    }

    fn outside_projects(self, input: &HookInput) -> Option<Reply> {
        let refusal = (self == Self::PreToolUse).then(|| input.refusal()).flatten()?;
        Some(Reply::new(self, Verdict::Deny(Denial::from(refusal)), None))
    }

    fn after_push(project: &Project, input: &HookInput) -> Option<Listed> {
        let failure =
            input.command()?.runs(Self::COMMIT).and_then(|()| project.push_current_branch()).and_then(Result::err);
        failure.or(input.session().and_then(|session| Watcher::watch_all(project, session).err()))
    }
}
