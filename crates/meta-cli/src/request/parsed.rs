mod option_name;
mod word;

use meta_listing::{Kind, Scope};
use meta_schema::{Actor, Entry, Id, Tag, Target, Task, Text};

use crate::request::ci::{Commit, SessionId};
use crate::request::cli_failure::{CliFailure, Invalid, Misuse};
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
        }
    }

    pub fn allowing(self, allowed: &[OptionName]) -> Result<Self, CliFailure> {
        self.given
            .iter()
            .find(|name| !allowed.contains(name))
            .map_or(Ok(()), |name| Err(CliFailure::raised::<Misuse>(format!("{name} does not apply here"))))?;
        Ok(self)
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

    pub fn previous_claimant(&self) -> Option<Actor> {
        self.from.clone()
    }

    pub fn scope(&self) -> Scope {
        self.scope
    }
}

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
        };
        empty.take(args)
    }
}
