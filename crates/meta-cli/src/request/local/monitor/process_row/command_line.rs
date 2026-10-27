use std::path::Path;
use std::str::FromStr;

use super::super::unreadable::Unreadable;
use crate::request::local::local_settings::SparedName;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommandLine(String);

impl CommandLine {
    const WRAPPER: [&'static str; 2] = ["-c", "source"];
    const SNAPSHOT: &'static str = "/.claude/shell-snapshots/snapshot-";

    pub fn spares(&self, name: &SparedName) -> Option<()> {
        self.0.split_whitespace().flat_map(|word| word.split('/')).any(|part| part == name.text()).then_some(())
    }

    pub fn names(&self, root: &Path) -> Option<()> {
        self.0.contains(&*root.to_string_lossy()).then_some(())
    }

    pub fn wrapper(&self) -> Option<()> {
        let words: Vec<&str> = self.0.split_whitespace().skip(1).take(Self::WRAPPER.len() + 1).collect();
        let (flags, snapshot) = words.split_at_checked(Self::WRAPPER.len())?;
        (flags == Self::WRAPPER && snapshot.iter().any(|path| path.contains(Self::SNAPSHOT))).then_some(())
    }

    pub fn words(&self) -> std::str::SplitWhitespace<'_> {
        self.0.split_whitespace()
    }
}

impl FromStr for CommandLine {
    type Err = Unreadable;

    fn from_str(command: &str) -> Result<Self, Self::Err> {
        Some(command.trim())
            .filter(|command| !command.is_empty())
            .map(|command| Self(command.to_owned()))
            .ok_or(Unreadable)
    }
}
