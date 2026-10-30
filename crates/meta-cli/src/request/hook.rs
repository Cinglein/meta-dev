mod call_phase;
mod hook_input;
mod linking;
mod reply;
#[cfg(test)]
mod tests;

use std::fmt;
use std::io::Read as _;
use std::panic::resume_unwind;
use std::str::FromStr;
use std::time::{Duration, Instant};

use chrono::Utc;
use meta_listing::Scope;
use meta_run::Budget;
use meta_schema::Actor;

use crate::request::ci::{SessionDir, Watcher};
use crate::request::cli_failure::{CliFailure, Invalid, Misuse};
use crate::request::output::{Listed, Output};
use crate::request::project::{CiSettings, Config, Project};
use crate::request::scale_policy::ScalePolicy;
pub use call_phase::CallPhase;
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
    const PUSH: &'static str = "push";
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

    pub fn budget(self, settings: &CiSettings) -> Budget {
        settings.budget(CiSettings::ROUTINE).within(Instant::now() + self.timeout().saturating_sub(Self::MARGIN))
    }

    pub fn respond(self) -> Result<Output, CliFailure> {
        let mut stdin = Vec::new();
        let read = std::io::stdin().read_to_end(&mut stdin).map_err(|failure| failure.to_string());
        let parsed = read.and_then(|_| serde_json::from_slice(&stdin).map_err(|failure| failure.to_string()));
        let input: HookInput = match parsed {
            Ok(input) => input,
            Err(failure) if self == Self::PreToolUse => {
                let reason = Listed::new(format!("cannot parse the hook input, so no command is allowed: {failure}"));
                return Ok(Output::Listed(Listed::new(self.refused(reason).to_string())));
            }
            Err(failure) => return Err(CliFailure::raised::<Invalid>(failure)),
        };
        let unstamped = self.phase().and_then(|call| input.stamp(call, Utc::now())).and_then(Result::err);
        let reply = match Project::found(input.cwd().as_path()) {
            Ok(Some(project)) => {
                let session = input.session().and_then(|session| SessionDir::for_session(&project, session));
                let budget = self.budget(project.ci_settings());
                Some(self.reply(&project, &input, budget).with_news(session.and_then(|dir| dir.news())))
            }
            Ok(None) => self.outside_projects(&input),
            Err(failure) => Some(self.refused(Listed::new(failure))),
        };
        let opened = self.opens_session().and_then(|()| input.open_session().err());
        let reply = reply.map(|reply| reply.with_news(opened));
        let reply = reply.or_else(|| unstamped.as_ref().map(|_| Reply::new(self, Verdict::Allow, None)));
        let reply = reply.map(|reply| reply.with_news(unstamped));
        Ok(Output::Listed(Listed::new(reply.map(|reply| reply.to_string()).unwrap_or_default())))
    }

    pub const fn phase(self) -> Option<CallPhase> {
        match self {
            Self::PreToolUse => Some(CallPhase::Started),
            Self::PostToolUse => Some(CallPhase::Finished),
            Self::SessionStart | Self::Stop => None,
        }
    }

    pub fn opens_session(self) -> Option<()> {
        (self == Self::SessionStart).then_some(())
    }

    pub fn refused(self, reason: Listed) -> Reply {
        match self {
            Self::PreToolUse => Reply::new(self, Verdict::Deny(Denial::from(reason)), None),
            Self::Stop => Reply::new(self, Verdict::Block(Blocking::from(reason)), None),
            Self::SessionStart | Self::PostToolUse => Reply::new(self, Verdict::Allow, Some(reason)),
        }
    }

    fn reply(self, project: &Project, input: &HookInput, budget: Budget) -> Reply {
        match self {
            Self::SessionStart => {
                let actor = input.announced_actor().unwrap_or_else(Actor::owner);
                let state = project
                    .log(budget)
                    .and_then(|log| Listed::from_log(&log, None, Scope::Open))
                    .map_or_else(|failure| format!("meta/log unreadable: {failure}\n"), |listed| listed.to_string());
                let shown = format!(
                    "state of meta/log. actor: {actor}. meta list | claim | done | release | wait | resume | link | issue | concern | open (issue, concern and open need --division <path>)\n{state}"
                );
                Reply::new(self, Verdict::Allow, Some(Listed::new(shown)))
            }
            Self::PreToolUse => Reply::new(
                self,
                Self::policy_refusal(project, input)
                    .or_else(|| input.refusal())
                    .or_else(|| input.unpushable(project, Self::PUSH))
                    .map(Denial::from)
                    .map_or(Verdict::Allow, Verdict::Deny),
                None,
            ),
            Self::PostToolUse => Self::after_tool_use(project, input, budget),
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

    fn policy_refusal(project: &Project, input: &HookInput) -> Option<Listed> {
        let policy = ScalePolicy::read(project.root()).ok().unwrap_or_default();
        Self::modal_refusal(input).or_else(|| {
            let lander = policy.queue().lander(input.announced_actor());
            input.command()?.refusal(project.config(), lander.as_ref())
        })
    }

    fn modal_refusal(input: &HookInput) -> Option<Listed> {
        ScalePolicy::embedded_section(ScalePolicy::tools)
            .map_or_else(|why| Some(Listed::new(why)), |tools| input.modal(tools))
    }

    fn outside_projects(self, input: &HookInput) -> Option<Reply> {
        let budget = CiSettings::try_from(&Config::default()).map(|settings| self.budget(&settings));
        let linked = budget.as_ref().ok().filter(|_| self == Self::PostToolUse);
        let linked = linked.and_then(|budget| Linking::after(input, *budget));
        let linking = linked.map(|linked| Reply::new(self, Verdict::Allow, Some(linked)));
        let refusal = (self == Self::PreToolUse).then(|| {
            Self::modal_refusal(input)
                .or_else(|| budget.map_or_else(|failure| Some(Listed::new(failure)), |_| input.refusal()))
        });
        let refusal = refusal.flatten();
        refusal.map(|refusal| Reply::new(self, Verdict::Deny(Denial::from(refusal)), None)).or(linking)
    }

    fn after_tool_use(project: &Project, input: &HookInput, budget: Budget) -> Reply {
        let (pushed, linked) = std::thread::scope(|scope| {
            let linked = scope.spawn(|| Linking::after(input, budget));
            (Self::after_push(project, input, budget), linked.join())
        });
        let linked = linked.unwrap_or_else(|panic| resume_unwind(panic));
        Reply::new(Self::PostToolUse, Verdict::Allow, pushed).with_news(linked)
    }

    fn after_push(project: &Project, input: &HookInput, budget: Budget) -> Option<Listed> {
        let command = input.command()?;
        let failure = command
            .runs(Self::COMMIT)
            .and_then(|()| input.unpushable(project, Self::COMMIT).or_else(|| project.push_failure(budget)));
        failure.or(input.session().and_then(|session| Watcher::watch_all(project, session, budget).err()))
    }
}
