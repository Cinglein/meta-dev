use std::fmt;

use crate::request::output::Listed;
use crate::request::project::{BranchName, Config};

#[derive(Debug)]
pub struct GitCall {
    words: Vec<String>,
}

impl GitCall {
    const VALUED_OPTIONS: &'static [&'static str] = &["-C", "-c"];

    pub fn parse(segment: &[String]) -> Option<Self> {
        let start = segment.iter().position(|word| word == "git")?;
        let (_git, rest) = segment.get(start..)?.split_first()?;
        Some(Self { words: Self::after_options(rest).to_vec() })
    }

    pub fn commit(&self) -> Option<()> {
        self.words.first().filter(|word| *word == "commit").map(|_| ())
    }

    pub fn push_refusal(&self, config: &Config) -> Option<Listed> {
        let args = self.words.split_first().filter(|(subcommand, _)| *subcommand == "push").map(|(_, args)| args)?;
        args.iter()
            .find(|word| Self::forces(word).is_some())
            .map(|word| Listed::new(format!("{self}: {word} forces; merge instead of rewriting history")))
            .or_else(|| self.protected(args, config))
    }

    fn protected(&self, args: &[String], config: &Config) -> Option<Listed> {
        let branch = Self::targets(args).find_map(|branch| config.protects(&branch).cloned())?;
        Some(Listed::new(format!("{self}: {branch} is protected; {}", config.push_fix(&branch))))
    }

    fn after_options(words: &[String]) -> &[String] {
        match words {
            [option, _, rest @ ..] if Self::VALUED_OPTIONS.contains(&option.as_str()) => Self::after_options(rest),
            [option, rest @ ..] if option.starts_with('-') => Self::after_options(rest),
            subcommand => subcommand,
        }
    }

    fn forces(word: &str) -> Option<&str> {
        (word == "--force"
            || word.starts_with('+')
            || (word.starts_with('-') && !word.starts_with("--") && word.contains('f')))
        .then_some(word)
    }

    fn targets(args: &[String]) -> impl Iterator<Item = BranchName> + '_ {
        args.iter()
            .filter(|word| !word.starts_with('-'))
            .filter_map(|word| word.rsplit(':').next())
            .filter_map(|target| target.parse().ok())
    }
}

impl fmt::Display for GitCall {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "git {}", self.words.join(" "))
    }
}
