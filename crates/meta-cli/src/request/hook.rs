mod hook_input;
mod linking;
mod reply;
#[cfg(test)]
mod tests;

use std::fmt;
use std::panic::resume_unwind;
use std::str::FromStr;
use std::time::{Duration, Instant};

use meta_listing::Scope;
use meta_schema::Actor;

use crate::request::ci::{SessionDir, Watcher};
use crate::request::cli_failure::{CliFailure, Invalid, Misuse};
use crate::request::output::{Listed, Output};
use crate::request::project::{CiSettings, Config, Project};
use hook_input::HookInput;
use linking::Linking;
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
    const SHORT: Duration = Duration::from_secs(10);
    const SESSION: Duration = Duration::from_secs(30);
    const LONG: Duration = Duration::from_secs(60);
    pub const MARGIN: Duration = Duration::from_secs(3);

    pub const fn timeout(self) -> Duration {
        match self {
            Self::SessionStart => Self::SESSION,
            Self::PreToolUse | Self::Stop => Self::SHORT,
            Self::PostToolUse => Self::LONG,
        }
    }

    pub fn git_deadline(self, settings: &CiSettings) -> Instant {
        let now = Instant::now();
        settings.git_deadline(now).min(now + self.timeout().saturating_sub(Self::MARGIN))
    }

    pub fn respond(self) -> Result<Output, CliFailure> {
        let stdin = std::io::read_to_string(std::io::stdin()).map_err(CliFailure::raised::<Invalid>)?;
        let input: HookInput = serde_json::from_str(&stdin).map_err(CliFailure::raised::<Invalid>)?;
        let reply = match Project::found(input.cwd().as_path()) {
            Ok(Some(project)) => {
                let session = input.session().and_then(|session| SessionDir::for_session(&project, session));
                Some(self.reply(&project, &input).with_news(session.and_then(|dir| dir.news())))
            }
            Ok(None) => self.outside_projects(&input),
            Err(failure) => Some(self.refused(Listed::new(failure))),
        };
        let opened = (self == Self::SessionStart).then(|| input.open_session()).and_then(Result::err);
        let reply = reply.map(|reply| reply.with_news(opened));
        Ok(Output::Listed(Listed::new(reply.map(|reply| reply.to_string()).unwrap_or_default())))
    }

    pub fn refused(self, reason: Listed) -> Reply {
        match self {
            Self::PreToolUse => Reply::new(self, Verdict::Deny(Denial::from(reason)), None),
            Self::Stop => Reply::new(self, Verdict::Block(Blocking::from(reason)), None),
            Self::SessionStart | Self::PostToolUse => Reply::new(self, Verdict::Allow, Some(reason)),
        }
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
                    "state of meta/log. actor: {actor}. meta list | claim | done | release | wait | resume | link | issue | concern | open (issue, concern and open need --division <path>)\n{state}"
                );
                Reply::new(self, Verdict::Allow, Some(Listed::new(shown)))
            }
            Self::PreToolUse => Reply::new(
                self,
                input
                    .command()
                    .and_then(|command| command.refusal(project.config()))
                    .or_else(|| input.refusal(self.git_deadline(project.ci_settings())))
                    .map(Denial::from)
                    .map_or(Verdict::Allow, Verdict::Deny),
                None,
            ),
            Self::PostToolUse => Self::after_tool_use(project, input),
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
        let linked = (self == Self::PostToolUse).then(|| Linking::after(input, None)).flatten();
        let linking = linked.map(|linked| Reply::new(self, Verdict::Allow, Some(linked)));
        let refused = |settings: CiSettings| input.refusal(self.git_deadline(&settings));
        let refusal = (self == Self::PreToolUse).then(|| {
            CiSettings::try_from(&Config::default()).map_or_else(|failure| Some(Listed::new(failure)), refused)
        });
        let refusal = refusal.flatten();
        refusal.map(|refusal| Reply::new(self, Verdict::Deny(Denial::from(refusal)), None)).or(linking)
    }

    fn after_tool_use(project: &Project, input: &HookInput) -> Reply {
        let (pushed, linked) = std::thread::scope(|scope| {
            let linked = scope.spawn(|| Linking::after(input, Some(project)));
            (Self::after_push(project, input), linked.join())
        });
        let linked = linked.unwrap_or_else(|panic| resume_unwind(panic));
        Reply::new(Self::PostToolUse, Verdict::Allow, pushed).with_news(linked)
    }

    fn after_push(project: &Project, input: &HookInput) -> Option<Listed> {
        let failure = input
            .command()?
            .runs(Self::COMMIT)
            .and_then(|()| project.push_failure(Self::PostToolUse.git_deadline(project.ci_settings())));
        failure.or(input.session().and_then(|session| Watcher::watch_all(project, session).err()))
    }
}
