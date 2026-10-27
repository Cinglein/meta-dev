use std::fmt;
use std::path::Path;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::super::unreadable::Unreadable;
use crate::request::local::local_settings::SparedName;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CommandLine(String);

impl CommandLine {
    const SHOWN_LEN: usize = 60;

    pub fn spares(&self, name: &SparedName) -> Option<()> {
        self.0.split_whitespace().flat_map(|word| word.split('/')).any(|part| part == name.text()).then_some(())
    }

    pub fn names(&self, root: &Path) -> Option<()> {
        self.0.contains(&*root.to_string_lossy()).then_some(())
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

impl fmt::Display for CommandLine {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0.chars().take(Self::SHOWN_LEN).collect::<String>())
    }
}
