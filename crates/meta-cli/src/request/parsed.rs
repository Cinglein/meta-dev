mod given;
mod missing_option;
mod option_name;

use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
mod word;

use meta_acceptance::{Gating, Observability};
use meta_listing::{Kind, Scope};
use meta_schema::{
    Acceptance, Actor, Blocker, Division, DueTime, Entry, Event, Id, LevelName, NewProject, NewTask, Project,
    PullRequest, Purpose, Scenario, Tag, Target, Task, TaskLink, TaskPurpose, Text, Tokens,
};

use crate::request::Request;
use crate::request::brief::BriefRequest;
use crate::request::ci::{Commit, SessionId};
use crate::request::cli_failure::{CliFailure, Invalid, Misuse};
use crate::request::merge_queue::MergeQueue;
use crate::request::rate::Rate;
use crate::request::relocate::Relocate;
use crate::request::status_view::StatusView;
use given::Given;
use missing_option::MissingOption;
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
    valued: Vec<Given>,
}

impl Parsed {
    const RESTORE: &'static str = "restore";
    const RUN: &'static str = "run";
    const PROPOSE: &'static str = "propose";
    const ACCEPT: &'static str = "accept";
    const LIST: &'static str = "list";

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
        name.excludes().iter().find(|other| self.given.contains(other)).map_or(Ok(()), |other| {
            Err(CliFailure::raised::<Misuse>(format!("{name} cannot be given after {other}")))
        })?;
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
            (
                OptionName::Instruments
                | OptionName::Serves
                | OptionName::CausedBy
                | OptionName::Project
                | OptionName::Root
                | OptionName::Row
                | OptionName::Span
                | OptionName::Gate,
                [value, rest @ ..],
            ) => Self { valued: [self.valued, vec![Given::new(name, Word::new(value))]].concat(), ..self }.take(rest),
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

    pub fn project_request(self) -> Result<Request, CliFailure> {
        let verb = self.words.first().map(ToString::to_string).unwrap_or_default();
        let rest = Self { words: self.words.iter().skip(1).cloned().collect(), ..self };
        match verb.as_str() {
            Self::PROPOSE => {
                Ok(Request::Write(Event::ProjectProposed(rest.allowing(&[OptionName::Division])?.proposal()?)))
            }
            Self::ACCEPT => {
                let options = [OptionName::Root, OptionName::Row, OptionName::Span, OptionName::Gate];
                Ok(Request::Write(Event::ProjectAccepted(rest.allowing(&options)?.accepted()?)))
            }
            Self::LIST => Ok(Request::List(Some(Kind::Projects), rest.allowing(&[])?.scope())),
            other => Err(CliFailure::raised::<Misuse>(format!("expected propose, accept or list, found `{other}`"))),
        }
    }

    fn each<Value: FromStr>(&self, name: OptionName) -> Result<Vec<Value>, CliFailure>
    where
        Value::Err: fmt::Display,
    {
        let wanted = self.valued.iter().filter_map(|given| given.value_of(name));
        wanted.map(|word| word.to_string().parse().map_err(CliFailure::raised::<Invalid>)).collect()
    }

    fn one<Value: FromStr>(&self, name: OptionName) -> Result<Option<Value>, CliFailure>
    where
        Value::Err: fmt::Display,
    {
        self.each(name).map(|values| values.into_iter().next())
    }

    pub fn purpose(&self) -> Result<Option<Purpose>, CliFailure> {
        let instruments = self.one::<Id<Project>>(OptionName::Instruments)?.map(Purpose::Instruments);
        let serves = self.one::<Id<Project>>(OptionName::Serves)?.map(Purpose::Serves);
        let caused = self.one(OptionName::CausedBy)?.map(|cause| Purpose::CausedBy { cause, project: None });
        let purpose = instruments.or(serves).or(caused);
        self.one(OptionName::Project)?.map_or(Ok(purpose), |project| {
            purpose
                .and_then(|purpose| purpose.in_project(project))
                .map(Some)
                .ok_or_else(|| CliFailure::raised::<Misuse>("--project goes with --caused-by"))
        })
    }

    pub fn purposing<Subject: FromStr>(
        &self,
        make: impl FnOnce(Subject, Option<Id<Project>>) -> Purpose,
    ) -> Result<TaskPurpose, CliFailure>
    where
        Subject::Err: fmt::Display,
    {
        match self.words.as_slice() {
            [task, subject] => Ok(TaskPurpose::new(
                task.to_string().parse().map_err(CliFailure::raised::<Invalid>)?,
                make(
                    subject.to_string().parse().map_err(CliFailure::raised::<Invalid>)?,
                    self.one(OptionName::Project)?,
                ),
            )),
            _ => {
                Err(CliFailure::raised::<Misuse>("expected a task id and what it serves, instruments or is caused by"))
            }
        }
    }

    pub fn proposal(&self) -> Result<NewProject, CliFailure> {
        Ok(NewProject::new(self.division()?, self.text()?))
    }

    pub fn accepted(&self) -> Result<Acceptance, CliFailure> {
        let missing = |option: MissingOption| CliFailure::raised::<Misuse>(option);
        let root = self.one(OptionName::Root)?.ok_or_else(|| missing(MissingOption::Root))?;
        let rows = Some(self.each(OptionName::Row)?).filter(|rows| !rows.is_empty());
        let spans = Some(self.each(OptionName::Span)?).filter(|spans| !spans.is_empty());
        let gates = Some(self.each(OptionName::Gate)?).filter(|gates| !gates.is_empty());
        let observability = Observability::named(
            rows.ok_or_else(|| missing(MissingOption::Row))?,
            spans.ok_or_else(|| missing(MissingOption::Span))?,
        )
        .map_err(CliFailure::raised::<Invalid>)?;
        let gating = Gating::try_from(gates.ok_or_else(|| missing(MissingOption::Gate))?)
            .map_err(CliFailure::raised::<Invalid>)?;
        match self.words.as_slice() {
            [project] => Ok(Acceptance::new(
                project.to_string().parse().map_err(CliFailure::raised::<Invalid>)?,
                root,
                observability,
                gating,
            )),
            _ => Err(CliFailure::raised::<Misuse>("expected one project id")),
        }
    }

    pub fn listable(self) -> Result<Self, CliFailure> {
        let parsed = self.allowing(&[OptionName::All])?;
        let refused = (parsed.kind()? == Some(Kind::Projects)).then(|| parsed.flagged(OptionName::All)).flatten();
        refused.map_or(Ok(parsed), |()| Err(CliFailure::raised::<Misuse>("projects list without --all")))
    }

    pub fn opened(&self) -> Result<NewTask, CliFailure> {
        let opened = NewTask::new(self.division()?, self.text()?, self.addresses());
        let sized = self.effort().into_iter().fold(opened, NewTask::allowing);
        let purposed = self.purpose()?.into_iter().fold(sized, NewTask::purposed);
        Ok(self.due().into_iter().fold(purposed, NewTask::due_by))
    }

    pub fn division(&self) -> Result<Division, CliFailure> {
        self.division.clone().ok_or_else(|| CliFailure::raised::<Misuse>(MissingOption::Division))
    }

    pub fn blocker(&self) -> Result<Blocker, CliFailure> {
        self.on.ok_or_else(|| CliFailure::raised::<Misuse>(MissingOption::On))
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
            valued: vec![],
        };
        empty.take(args)
    }
}
