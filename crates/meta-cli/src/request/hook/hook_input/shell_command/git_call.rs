mod force;

use std::fmt;

use crate::request::output::Listed;
use crate::request::project::{BranchName, Config};
use crate::request::scale_policy::{Refused, ShellRules};
use force::Force;

#[derive(Debug)]
pub struct GitCall {
    directories: Vec<String>,
    words: Vec<String>,
}

impl GitCall {
    const VALUED_OPTIONS: &'static [&'static str] =
        &["-C", "-c", "--git-dir", "--work-tree", "--namespace", "--config-env"];
    const PROGRAM: &'static str = "git";
    const PUSH: &'static str = "push";
    const DIRECTORY: &'static str = "-C";
    const OPTION_PAIR_LEN: usize = 2;
    const COMMIT_WRITERS: &'static [&'static str] =
        &["commit", "rebase", "cherry-pick", "merge", "revert", "pull", "am", "commit-tree"];
    const ABORT: &'static str = "--abort";

    pub fn parse(segment: &[String]) -> Option<Self> {
        let start = segment.iter().position(|word| word.rsplit('/').next() == Some(Self::PROGRAM))?;
        let (_git, rest) = segment.get(start..)?.split_first()?;
        let words = Self::after_options(rest);
        let directories = rest
            .get(..rest.len() - words.len())?
            .windows(Self::OPTION_PAIR_LEN)
            .filter_map(|pair| match pair {
                [option, directory] if option == Self::DIRECTORY => Some(directory.clone()),
                _ => None,
            })
            .collect();
        Some(Self { directories, words: words.to_vec() })
    }

    pub fn writes_commits(&self) -> Option<()> {
        let (subcommand, args) = self.words.split_first()?;
        (Self::COMMIT_WRITERS.contains(&subcommand.as_str()) && !args.iter().any(|arg| arg == Self::ABORT))
            .then_some(())
    }

    pub fn breach(&self, rules: &ShellRules) -> Option<Refused> {
        let (subcommand, args) = self.words.split_first()?;
        let forced = args.iter().find(|arg| subcommand == Self::PUSH && Force::in_word(arg).is_some());
        rules.git().breach(subcommand).or_else(|| forced.map(|arg| Refused::spelled(&[Self::PROGRAM, Self::PUSH, arg])))
    }

    pub fn nested(&self, rules: &ShellRules) -> Option<String> {
        let (subcommand, args) = self.words.split_first()?;
        rules.git().nested(subcommand, args)
    }

    pub fn directory_words(&self) -> &[String] {
        &self.directories
    }

    pub fn runs(&self, subcommand: &str) -> Option<()> {
        self.words.first().filter(|word| *word == subcommand).map(|_| ())
    }

    pub fn push_refusal(&self, config: &Config) -> Option<Listed> {
        let args =
            self.words.split_first().filter(|(subcommand, _)| *subcommand == Self::PUSH).map(|(_, args)| args)?;
        args.iter()
            .find(|word| Force::in_word(word) == Some(Force::Hard))
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
