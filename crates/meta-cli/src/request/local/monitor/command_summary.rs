use std::fmt;

use serde::{Deserialize, Serialize};

use super::process_row::CommandLine;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CommandSummary(String);

impl CommandSummary {
    const SHOWN_LEN: usize = 48;
}

impl From<&CommandLine> for CommandSummary {
    fn from(command: &CommandLine) -> Self {
        let words: Vec<&str> =
            command.words().map(|word| word.rsplit('/').find(|part| !part.is_empty()).unwrap_or(word)).collect();
        Self(words.join(" ").chars().take(Self::SHOWN_LEN).collect())
    }
}

impl fmt::Display for CommandSummary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
