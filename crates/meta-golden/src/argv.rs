use std::fmt;
use std::str::FromStr;

use meta_run::Tool;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Argv(Vec<String>);

impl Argv {
    #[must_use]
    pub fn calls<Call: Tool>(&self, call: Call) -> bool {
        let words = call.words().iter().copied();
        self.0.first().is_some_and(|program| program == Call::PROGRAM)
            && self.0.iter().skip(1).map(String::as_str).take(call.words().len()).eq(words)
    }

    #[must_use]
    pub fn answers(&self, asked: &Self) -> bool {
        self.0.get(1..) == asked.0.get(1..)
    }
}

impl From<Vec<String>> for Argv {
    fn from(words: Vec<String>) -> Self {
        Self(words)
    }
}

impl FromStr for Argv {
    type Err = serde_json::Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(text)
    }
}

impl fmt::Display for Argv {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&serde_json::to_string(self).map_err(|_| fmt::Error)?)
    }
}
