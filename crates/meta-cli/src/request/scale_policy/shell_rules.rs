mod program;

use std::collections::BTreeMap;

use program::Program;
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShellRules {
    #[serde(default)]
    forbidden: Vec<Program>,
    #[serde(default)]
    wrappers: Vec<Program>,
    #[serde(default)]
    interpreters: Vec<Program>,
    #[serde(default)]
    valued: BTreeMap<String, Vec<Program>>,
    #[serde(default)]
    forbidden_git: Vec<String>,
    #[serde(default)]
    git_flags: BTreeMap<String, Vec<String>>,
}

impl ShellRules {
    const QUERIED: &'static str = "command";
    const QUERIES: &'static [&'static str] = &["-v", "-V"];
    const EVAL: &'static str = "eval";
    const GIT: &'static str = "git";
    const HERE_STRING: &'static str = "<<<";
    const COMMAND_FLAG: char = 'c';

    pub fn union(mut self, other: Self) -> Self {
        self.forbidden.extend(other.forbidden);
        self.wrappers.extend(other.wrappers);
        self.interpreters.extend(other.interpreters);
        self.forbidden_git.extend(other.forbidden_git);
        for (wrapper, flags) in other.valued {
            self.valued.entry(wrapper).or_default().extend(flags);
        }
        for (subcommand, flags) in other.git_flags {
            self.git_flags.entry(subcommand).or_default().extend(flags);
        }
        self
    }

    pub fn git_in<'a>(&self, segment: &'a [String]) -> Option<&'a [String]> {
        let rest = segment.get(self.command_at(segment)?..)?;
        rest.first().filter(|word| word.rsplit('/').next() == Some(Self::GIT)).map(|_| rest)
    }

    pub fn git_breach(&self, subcommand: &str, args: &[String]) -> Option<String> {
        let flags = self.git_flags.get(subcommand);
        let flagged = flags.and_then(|flags| args.iter().find(|arg| flags.iter().any(|flag| Self::flagged(arg, flag))));
        let named = self.forbidden_git.iter().any(|word| word == subcommand).then(|| subcommand.to_owned());
        named.or_else(|| flagged.map(|arg| format!("{subcommand} {arg}")))
    }

    fn flagged(arg: &str, flag: &str) -> bool {
        match flag.strip_prefix("--") {
            Some(_) => arg == flag || arg.strip_prefix(flag).is_some_and(|rest| rest.starts_with('=')),
            None => arg.starts_with('-') && !arg.starts_with("--") && flag.chars().skip(1).all(|c| arg.contains(c)),
        }
    }

    pub fn forbidden_in<'a>(&self, segment: &'a [String]) -> Option<&'a str> {
        let word = segment.get(self.command_at(segment)?)?.as_str();
        self.forbidden.iter().any(|program| program.names(word)).then_some(word)
    }

    pub fn stdin_shell<'a>(&self, segment: &'a [String]) -> Option<&'a str> {
        let (word, args) = self.interpreter(segment)?;
        let script = args.iter().take_while(|arg| arg.as_str() != Self::HERE_STRING).any(|arg| !arg.starts_with('-'));
        (word != Self::EVAL && Self::command_payload(args).is_none() && !script).then_some(word.as_str())
    }

    pub fn nested(&self, segment: &[String]) -> Option<String> {
        let (word, args) = self.interpreter(segment)?;
        Self::command_payload(args).or_else(|| (word == Self::EVAL).then(|| args.join(" ")))
    }

    fn interpreter<'a>(&self, segment: &'a [String]) -> Option<(&'a String, &'a [String])> {
        let (word, args) = segment.get(self.command_at(segment)?..)?.split_first()?;
        self.interpreters.iter().find(|program| program.names(word))?;
        Some((word, args))
    }

    fn command_payload(args: &[String]) -> Option<String> {
        let flag = args
            .iter()
            .position(|arg| arg.starts_with('-') && !arg.starts_with("--") && arg.contains(Self::COMMAND_FLAG))?;
        args.get(flag + 1).cloned()
    }

    fn takes_value(&self, wrapper: &str, flag: &str) -> bool {
        self.valued.get(wrapper).is_some_and(|flags| flags.iter().any(|valued| valued.names(flag)))
    }

    fn command_at(&self, segment: &[String]) -> Option<usize> {
        let mut words = segment.iter().enumerate();
        let mut wrapper: Option<&str> = None;
        while let Some((index, word)) = words.next() {
            let option = word.starts_with('-') || word.starts_with(|c: char| c.is_ascii_digit());
            if self.wrappers.iter().any(|wrapped| wrapped.names(word)) {
                let query = segment.get(index + 1).is_some_and(|next| Self::QUERIES.contains(&next.as_str()));
                if word == Self::QUERIED && query {
                    return None;
                }
                wrapper = Some(word.rsplit('/').next().unwrap_or(word));
            } else if wrapper.is_some_and(|name| self.takes_value(name, word)) {
                words.next();
            } else if !((wrapper.is_some() && option) || word.contains('=')) {
                return Some(index);
            }
        }
        None
    }
}
