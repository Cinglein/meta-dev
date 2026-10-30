mod git_breach;
mod git_rules;
mod program;
mod programs;
mod table;
mod union;

use serde::Deserialize;

pub use git_breach::GitBreach;
pub use git_rules::GitRules;
use program::Program;
use programs::Programs;
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
    #[serde(default)]
    git: GitRules,
}

impl ShellRules {
    const QUERIED: &'static str = "command";
    const EVAL: &'static str = "eval";
    const GIT: &'static str = "git";
    const HERE_STRING: &'static str = "<<<";
    const COMMAND_FLAG: char = 'c';

    pub fn forbidden_in<'segment>(&self, segment: &'segment [String]) -> Option<&'segment String> {
        let word = self.command(segment, None)?.first()?;
        self.forbidden.names(word).then_some(word)
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
        let named = |arg: &&String| arg.as_str() != Self::HERE_STRING;
        (word != Self::EVAL
            && Self::command_payload(args).is_none()
            && !args.iter().take_while(named).any(|arg| !arg.starts_with('-')))
        .then_some(word.as_str())
    }

    pub fn nested(&self, segment: &[String]) -> Option<String> {
        let (word, args) =
            self.command(segment, None)?.split_first().filter(|(word, _)| self.interpreters.names(word))?;
        Self::command_payload(args).or_else(|| (word == Self::EVAL).then(|| args.join(" ")))
    }

    fn command_payload(args: &[String]) -> Option<String> {
        let flagged =
            |arg: &&String| arg.starts_with('-') && !arg.starts_with("--") && arg.contains(Self::COMMAND_FLAG);
        args.iter().skip_while(|arg| !flagged(arg)).nth(1).cloned()
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
            git: self.git.union(other.git),
        }
    }
}
