mod option_name;

use std::path::PathBuf;
mod word;

use meta_listing::{Kind, Scope};
use meta_schema::{
    Actor, Blocker, Claim, Division, DueTime, Entry, Id, LevelName, PullRequest, Scenario, Tag, Target, Task, TaskLink,
    Text, Tokens,
};

use crate::request::brief::BriefRequest;
use crate::request::ci::{Commit, SessionId};
use crate::request::cli_failure::{CliFailure, Invalid, Misuse};
use crate::request::dispatch::Dispatch;
use crate::request::merge_queue::MergeQueue;
use crate::request::rate::Rate;
use crate::request::relocate::Relocate;
use crate::request::status_view::StatusView;
pub use option_name::OptionName;
use word::Word;

#[derive(Debug)]
pub struct Parsed {
    words: Vec<Word>,
    given: Vec<OptionName>,
    source: Option<Text>,
    tags: Vec<Tag>,
    addresses: Vec<Target>,
    from: Option<Actor>,
    scope: Scope,
    division: Option<Division>,
    effort: Option<Tokens>,
    due: Option<DueTime>,
    on: Option<Blocker>,
    view: StatusView,
    severity: Option<LevelName>,
    scenario: Option<Scenario>,
    actor: Option<Actor>,
    scratch: Option<Text>,
    agent: Option<Actor>,
}

impl Parsed {
    const RESTORE: &'static str = "restore";
    const RUN: &'static str = "run";

    fn take(self, args: &[String]) -> Result<Self, CliFailure> {
        match args {
            [] => Ok(self),
            [separator, rest @ ..] if separator == "--" => {
                Ok(Self { words: [self.words, rest.iter().map(Word::new).collect()].concat(), ..self })
            }
            [option, rest @ ..] if option.starts_with("--") => self.option(option.parse()?, rest),
            [word, rest @ ..] => Self { words: [self.words, vec![Word::new(word)]].concat(), ..self }.take(rest),
        }
    }

    fn option(mut self, name: OptionName, args: &[String]) -> Result<Self, CliFailure> {
        self.given.push(name);
        match (name, args) {
            (OptionName::All, rest) => Self { scope: Scope::All, ..self }.take(rest),
            (OptionName::Flags, rest) => Self { view: StatusView::Flags, ..self }.take(rest),
            (OptionName::Worktree | OptionName::DryRun, rest) => self.take(rest),
            (_, []) => Err(CliFailure::raised::<Misuse>(format!("{name} needs a value"))),
            (OptionName::Source, [value, rest @ ..]) => {
                Self { source: Some(value.parse().map_err(CliFailure::raised::<Invalid>)?), ..self }.take(rest)
            }
            (OptionName::Tag, [value, rest @ ..]) => {
                let tag = value.parse().map_err(CliFailure::raised::<Invalid>)?;
                Self { tags: [self.tags, vec![tag]].concat(), ..self }.take(rest)
            }
            (OptionName::Addresses, [value, rest @ ..]) => {
                let target = value.parse().map_err(CliFailure::raised::<Invalid>)?;
                Self { addresses: [self.addresses, vec![target]].concat(), ..self }.take(rest)
            }
            (OptionName::From, [value, rest @ ..]) => {
                Self { from: Some(value.parse().map_err(CliFailure::raised::<Invalid>)?), ..self }.take(rest)
            }
            (OptionName::Agent, [value, rest @ ..]) => {
                Self { agent: Some(value.parse().map_err(CliFailure::raised::<Invalid>)?), ..self }.take(rest)
            }
            (OptionName::Division, [value, rest @ ..]) => {
                Self { division: Some(value.parse().map_err(CliFailure::raised::<Invalid>)?), ..self }.take(rest)
            }
            (OptionName::Effort, [value, rest @ ..]) => {
                Self { effort: Some(value.parse().map_err(CliFailure::raised::<Invalid>)?), ..self }.take(rest)
            }
            (OptionName::Due, [value, rest @ ..]) => {
                Self { due: Some(value.parse().map_err(CliFailure::raised::<Invalid>)?), ..self }.take(rest)
            }
            (OptionName::On, [value, rest @ ..]) => {
                Self { on: Some(value.parse().map_err(CliFailure::raised::<Invalid>)?), ..self }.take(rest)
            }
            (OptionName::Severity, [value, rest @ ..]) => {
                Self { severity: Some(value.parse().map_err(CliFailure::raised::<Invalid>)?), ..self }.take(rest)
            }
            (OptionName::Actor, [value, rest @ ..]) => {
                Self { actor: Some(value.parse().map_err(CliFailure::raised::<Invalid>)?), ..self }.take(rest)
            }
            (OptionName::Scratch, [value, rest @ ..]) => {
                Self { scratch: Some(value.parse().map_err(CliFailure::raised::<Invalid>)?), ..self }.take(rest)
            }
            (OptionName::Scenario, [value, rest @ ..]) => {
                let scenario = value.parse::<Text>().map_err(CliFailure::raised::<Invalid>)?;
                Self { scenario: Some(Scenario::from(scenario)), ..self }.take(rest)
            }
        }
    }

    pub fn allowing(self, allowed: &[OptionName]) -> Result<Self, CliFailure> {
        self.given
            .iter()
            .find(|name| !allowed.contains(name))
            .map_or(Ok(()), |name| Err(CliFailure::raised::<Misuse>(format!("{name} does not apply here"))))?;
        Ok(self)
    }

    pub fn flagged(&self, name: OptionName) -> Option<()> {
        self.given.contains(&name).then_some(())
    }

    pub fn text(&self) -> Result<Text, CliFailure> {
        let joined: Vec<String> = self.words.iter().map(ToString::to_string).collect();
        joined.join(" ").parse().map_err(CliFailure::raised::<Invalid>)
    }

    pub fn task(&self) -> Result<Id<Task>, CliFailure> {
        match self.words.as_slice() {
            [word] => word.to_string().parse().map_err(CliFailure::raised::<Invalid>),
            _ => Err(CliFailure::raised::<Misuse>("expected one task id")),
        }
    }

    pub fn task_link(&self) -> Result<TaskLink, CliFailure> {
        match self.words.as_slice() {
            [task, pull_request] => Ok(TaskLink::new(
                task.to_string().parse().map_err(CliFailure::raised::<Invalid>)?,
                pull_request.to_string().parse::<PullRequest>().map_err(CliFailure::raised::<Invalid>)?,
            )),
            _ => Err(CliFailure::raised::<Misuse>("expected a task id and a pull request number")),
        }
    }

    pub fn division(&self) -> Result<Division, CliFailure> {
        self.division.clone().ok_or_else(|| CliFailure::raised::<Misuse>("--division <path> is required"))
    }

    pub fn blocker(&self) -> Result<Blocker, CliFailure> {
        self.on.ok_or_else(|| CliFailure::raised::<Misuse>("--on <issue id|task id|review|ci|merge> is required"))
    }

    pub fn severity(&self) -> Option<LevelName> {
        self.severity.clone()
    }

    pub fn rating(&self) -> Result<Rate, CliFailure> {
        match self.words.as_slice() {
            [item, level] => Ok(Rate::new(
                item.to_string().parse().map_err(CliFailure::raised::<Invalid>)?,
                level.to_string().parse().map_err(CliFailure::raised::<Invalid>)?,
                self.scenario.clone(),
            )),
            _ => Err(CliFailure::raised::<Misuse>("expected a concern or issue id and a level")),
        }
    }

    pub fn relocation(&self) -> Result<Relocate, CliFailure> {
        match self.words.as_slice() {
            [_] if self.severity.is_none() && self.scenario.is_some() => {
                Err(CliFailure::raised::<Misuse>("--scenario needs --severity"))
            }
            [item] => Ok(Relocate::new(
                item.to_string().parse().map_err(CliFailure::raised::<Invalid>)?,
                self.division()?,
                self.severity(),
                self.scenario.clone(),
            )),
            _ => Err(CliFailure::raised::<Misuse>("expected one concern or issue id")),
        }
    }

    pub fn effort(&self) -> Option<Tokens> {
        self.effort
    }

    pub fn due(&self) -> Option<DueTime> {
        self.due
    }

    pub fn session_and_commit<Built>(
        &self,
        build: impl FnOnce(SessionId, Commit) -> Built,
    ) -> Result<Built, CliFailure> {
        match self.words.as_slice() {
            [session, commit] => Ok(build(
                session.to_string().parse().map_err(CliFailure::raised::<Invalid>)?,
                commit.to_string().parse().map_err(CliFailure::raised::<Invalid>)?,
            )),
            _ => Err(CliFailure::raised::<Misuse>("expected a session id and a commit")),
        }
    }

    pub fn entry_and_text<Built>(&self, build: impl FnOnce(Entry, Text) -> Built) -> Result<Built, CliFailure> {
        let (entry, text) =
            self.words.split_first().ok_or_else(|| CliFailure::raised::<Misuse>("expected an id and text"))?;
        let joined: Vec<String> = text.iter().map(ToString::to_string).collect();
        Ok(build(
            entry.to_string().parse().map_err(CliFailure::raised::<Invalid>)?,
            joined.join(" ").parse().map_err(CliFailure::raised::<Invalid>)?,
        ))
    }

    pub fn kind(&self) -> Result<Option<Kind>, CliFailure> {
        match self.words.as_slice() {
            [] => Ok(None),
            [word] => word
                .to_string()
                .parse()
                .map(Some)
                .map_err(|_| CliFailure::raised::<Misuse>(format!("unknown kind `{word}`"))),
            _ => Err(CliFailure::raised::<Misuse>("expected at most one kind")),
        }
    }

    pub fn source(&self) -> Option<Text> {
        self.source.clone()
    }

    pub fn tags(&self) -> Vec<Tag> {
        self.tags.clone()
    }

    pub fn addresses(&self) -> Vec<Target> {
        self.addresses.clone()
    }

    pub fn restored(&self) -> Option<PathBuf> {
        match self.words.as_slice() {
            [verb, path] if verb.to_string() == Self::RESTORE => Some(PathBuf::from(path.to_string())),
            _ => None,
        }
    }

    pub fn brief(&self) -> Result<BriefRequest, CliFailure> {
        Ok(BriefRequest::new(self.task()?, self.actor.clone(), self.scratch.clone()))
    }

    pub fn queue(&self) -> Result<MergeQueue, CliFailure> {
        match self.words.as_slice() {
            [verb, pulls @ ..] if verb.to_string() == Self::RUN => MergeQueue::new(
                pulls
                    .iter()
                    .map(|pull| pull.to_string().parse().map_err(CliFailure::raised::<Invalid>))
                    .collect::<Result<_, _>>()?,
            ),
            _ => Err(CliFailure::raised::<Misuse>("expected `queue run <pr-number>...`")),
        }
    }

    pub fn previous_claimant(&self) -> Option<Actor> {
        self.from.clone()
    }

    pub fn dispatch(&self, claim: Claim) -> Result<Dispatch, Claim> {
        self.agent.clone().map(|agent| Dispatch::new(claim.clone(), agent)).ok_or(claim)
    }

    pub fn scope(&self) -> Scope {
        self.scope
    }

    pub fn view(&self) -> StatusView {
        self.view
    }
}

#[cfg(test)]
mod tests;

impl TryFrom<&[String]> for Parsed {
    type Error = CliFailure;

    fn try_from(args: &[String]) -> Result<Self, Self::Error> {
        let empty = Self {
            words: vec![],
            given: vec![],
            source: None,
            tags: vec![],
            addresses: vec![],
            from: None,
            scope: Scope::Open,
            division: None,
            effort: None,
            due: None,
            on: None,
            view: StatusView::Tree,
            severity: None,
            scenario: None,
            actor: None,
            scratch: None,
            agent: None,
        };
        empty.take(args)
    }
}
