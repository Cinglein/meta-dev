mod flag;
mod git_rules;
mod position;
mod program;
mod programs;
mod refused;
mod table;
mod union;

use flag::Flag;
use serde::Deserialize;

pub use git_rules::GitRules;
pub use position::Position;
use program::Program;
use programs::Programs;
pub use refused::Refused;
use table::Table;
pub use union::Union;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShellRules {
    #[serde(default)]
    forbidden: Programs,
    #[serde(default)]
    wrappers: Programs,
    #[serde(default)]
    interpreters: Programs,
    #[serde(default)]
    valued: Table<Program>,
    #[serde(default, rename = "nested")]
    commanding: Table<Flag>,
    #[serde(default)]
    flags: Table<Flag>,
    #[serde(default)]
    git: GitRules,
}

impl ShellRules {
    const QUERIED: &'static str = "command";
    const EVAL: &'static str = "eval";
    const GIT: &'static str = "git";

    pub fn forbidden_in(&self, segment: &[String]) -> Option<Refused> {
        let (word, args) = self.command(segment, None)?.split_first()?;
        let flags = self.flags.words_for(Program::base(word));
        let flagged = flags.iter().find_map(|flag| args.iter().find(|arg| flag.names(arg)));
        self.forbidden
            .names(word)
            .then(|| Refused::spelled(&[word]))
            .or_else(|| flagged.map(|arg| Refused::spelled(&[word, arg])))
    }

    pub fn program_at(&self, segment: &[String]) -> Option<Position> {
        self.command(segment, None).map(|rest| Position::between(segment, rest))
    }

    pub fn git_in<'segment>(&self, segment: &'segment [String]) -> Option<&'segment [String]> {
        self.command(segment, None).filter(|rest| rest.first().is_some_and(|word| Program::base(word) == Self::GIT))
    }

    pub fn git(&self) -> &GitRules {
        &self.git
    }

    pub fn stdin_shell<'segment>(&self, segment: &'segment [String]) -> Option<&'segment str> {
        let (word, args) =
            self.command(segment, None)?.split_first().filter(|(word, _)| self.interpreters.names(word))?;
        (word != Self::EVAL && Program::payload(args).is_none() && !args.iter().any(|arg| !arg.starts_with('-')))
            .then_some(word.as_str())
    }

    pub fn nested(&self, segment: &[String]) -> Option<String> {
        let interpreter = self.command(segment, None).and_then(<[String]>::split_first);
        let interpreted = interpreter
            .filter(|(word, _)| self.interpreters.names(word))
            .and_then(|(word, args)| Program::payload(args).or_else(|| (word == Self::EVAL).then(|| args.join(" "))));
        interpreted.or_else(|| self.commanding.command_in(segment))
    }

    fn takes_value(&self, wrapper: &str, flag: &str) -> bool {
        self.valued.words_for(wrapper).iter().any(|valued| valued.names(flag))
    }

    fn command<'segment>(&self, words: &'segment [String], wrapper: Option<&str>) -> Option<&'segment [String]> {
        let (word, rest) = words.split_first()?;
        match word {
            word if self.wrappers.names(word) && word == Self::QUERIED && Program::queries(rest.first()) => None,
            word if self.wrappers.names(word) => self.command(rest, Some(Program::base(word))),
            word if wrapper.is_some_and(|name| self.takes_value(name, word)) => {
                rest.get(1..).and_then(|after| self.command(after, wrapper))
            }
            word if wrapper.is_some() && Program::option(word) || word.contains('=') => self.command(rest, wrapper),
            _ => Some(words),
        }
    }
}

impl Union for ShellRules {
    fn union(self, other: Self) -> Self {
        Self {
            forbidden: self.forbidden.union(other.forbidden),
            wrappers: self.wrappers.union(other.wrappers),
            interpreters: self.interpreters.union(other.interpreters),
            valued: self.valued.union(other.valued),
            commanding: self.commanding.union(other.commanding),
            flags: self.flags.union(other.flags),
            git: self.git.union(other.git),
        }
    }
}
