mod cli_failure;
mod output;
mod parsed;
mod project;

use meta_log::{LogFailure, LogStore};
use meta_schema::{Actor, Claim, Event, NewConcern, NewIssue, NewTask, Record, TaskRef};
use meta_state::{Kind, Listing, Scope, State, Summary};

use cli_failure::CliFailure;
use output::{Listed, Output};
use parsed::{OptionName, Parsed};
use project::Project;

#[derive(Debug)]
pub enum Command {
    Write(Event),
    List(Option<Kind>, Scope),
}

impl TryFrom<Vec<String>> for Command {
    type Error = CliFailure;

    fn try_from(args: Vec<String>) -> Result<Self, Self::Error> {
        let (_program, words) = args.split_first().ok_or_else(|| CliFailure::usage("missing program name"))?;
        let (name, rest) = words.split_first().ok_or_else(|| CliFailure::usage("missing command"))?;
        let parsed = Parsed::new(rest)?;
        let task = || parsed.task().map(TaskRef::new);
        match name.as_str() {
            "concern" => {
                let parsed = parsed.allowing(&[OptionName::Source, OptionName::Tag])?;
                Ok(Self::Write(Event::ConcernIdentified(NewConcern::new(
                    parsed.text()?,
                    parsed.source(),
                    parsed.tags(),
                ))))
            }
            "issue" => Ok(Self::Write(Event::IssueReported(NewIssue::new(parsed.allowing(&[])?.text()?)))),
            "open" => {
                let parsed = parsed.allowing(&[OptionName::Addresses])?;
                Ok(Self::Write(Event::TaskOpened(NewTask::new(parsed.text()?, parsed.addresses()))))
            }
            "claim" => {
                let parsed = parsed.allowing(&[OptionName::From])?;
                Ok(Self::Write(Event::TaskClaimed(Claim::new(parsed.task()?, parsed.previous_claimant()))))
            }
            "release" => Ok(Self::Write(Event::TaskReleased(task()?))),
            "done" => Ok(Self::Write(Event::TaskDone(task()?))),
            "reopen" => Ok(Self::Write(Event::TaskReopened(task()?))),
            "list" => Ok(Self::List(parsed.kind()?, parsed.allowing(&[OptionName::All])?.scope())),
            other => Err(CliFailure::usage(format!("unknown command `{other}`"))),
        }
    }
}

impl Command {
    pub fn answer(self) -> Result<Output, CliFailure> {
        let log = Project::discover()?.log()?;
        match self {
            Self::Write(event) => {
                let actor = Self::actor()?;
                let record = log.append(&mut |snapshot| {
                    let record = Record::new(actor.clone(), event.clone());
                    State::fold(snapshot.complete()?).vet(&record).map_err(LogFailure::refused)?;
                    Ok(record)
                })?;
                Ok(Output::Written(record))
            }
            Self::List(kind, scope) => {
                let snapshot = log.read()?;
                let state = State::fold(snapshot.records());
                let listed = kind.map_or_else(
                    || Listed::new(format!("{}{snapshot}", Summary::new(&state))),
                    |kind| Listed::new(format!("{}{snapshot}", Listing::new(&state, kind, scope))),
                );
                Ok(Output::Listed(listed))
            }
        }
    }

    fn actor() -> Result<Actor, CliFailure> {
        ["META_ACTOR", "USER"]
            .iter()
            .find_map(|variable| std::env::var(variable).ok().filter(|name| !name.is_empty()))
            .ok_or_else(|| CliFailure::config("set META_ACTOR"))?
            .parse()
            .map_err(CliFailure::invalid)
    }
}
