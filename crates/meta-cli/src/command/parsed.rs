mod option_name;
mod word;

use std::fmt;

use meta_schema::{Actor, Id, Tag, Target, Task, Text};
use meta_state::{Kind, Scope};

use crate::command::ci::{Commit, SessionId};
use crate::command::cli_failure::CliFailure;
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
}

impl Parsed {
    pub fn new(args: &[String]) -> Result<Self, CliFailure> {
        let empty = Self {
            words: vec![],
            given: vec![],
            source: None,
            tags: vec![],
            addresses: vec![],
            from: None,
            scope: Scope::Open,
        };
        empty.take(args)
    }

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
            (_, []) => Err(CliFailure::usage(format!("{name} needs a value"))),
            (OptionName::Source, [value, rest @ ..]) => {
                Self { source: Some(value.parse().map_err(CliFailure::invalid)?), ..self }.take(rest)
            }
            (OptionName::Tag, [value, rest @ ..]) => {
                let tag = value.parse().map_err(CliFailure::invalid)?;
                Self { tags: [self.tags, vec![tag]].concat(), ..self }.take(rest)
            }
            (OptionName::Addresses, [value, rest @ ..]) => {
                let target = value.parse().map_err(CliFailure::invalid)?;
                Self { addresses: [self.addresses, vec![target]].concat(), ..self }.take(rest)
            }
            (OptionName::From, [value, rest @ ..]) => {
                Self { from: Some(value.parse().map_err(CliFailure::invalid)?), ..self }.take(rest)
            }
        }
    }

    pub fn allowing(self, allowed: &[OptionName]) -> Result<Self, CliFailure> {
        self.given
            .iter()
            .find(|name| !allowed.contains(name))
            .map_or(Ok(()), |name| Err(CliFailure::usage(format!("{name} does not apply here"))))?;
        Ok(self)
    }

    pub fn text(&self) -> Result<Text, CliFailure> {
        let joined: Vec<String> = self.words.iter().map(ToString::to_string).collect();
        joined.join(" ").parse().map_err(CliFailure::invalid)
    }

    pub fn task(&self) -> Result<Id<Task>, CliFailure> {
        match self.words.as_slice() {
            [word] => word.to_string().parse().map_err(CliFailure::invalid),
            _ => Err(CliFailure::usage("expected one task id")),
        }
    }

    pub fn session_and_commit<Built>(
        &self,
        build: impl FnOnce(SessionId, Commit) -> Built,
    ) -> Result<Built, CliFailure> {
        match self.words.as_slice() {
            [session, commit] => Ok(build(
                session.to_string().parse().map_err(CliFailure::invalid)?,
                commit.to_string().parse().map_err(CliFailure::invalid)?,
            )),
            _ => Err(CliFailure::usage("expected a session id and a commit")),
        }
    }

    pub fn kind(&self) -> Result<Option<Kind>, CliFailure> {
        match self.words.as_slice() {
            [] => Ok(None),
            [word] => word.to_string().parse().map(Some).map_err(CliFailure::usage),
            _ => Err(CliFailure::usage("expected at most one kind")),
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

    pub fn previous_claimant(&self) -> Option<Actor> {
        self.from.clone()
    }

    pub fn scope(&self) -> Scope {
        self.scope
    }
}

impl fmt::Display for Parsed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.words.iter().try_for_each(|word| write!(formatter, "{word} "))
    }
}
