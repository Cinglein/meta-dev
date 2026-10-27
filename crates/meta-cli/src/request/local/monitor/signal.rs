use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Signal {
    Term,
    Kill,
}

impl Signal {
    pub fn flag(self) -> &'static str {
        match self {
            Self::Term => "-TERM",
            Self::Kill => "-KILL",
        }
    }
}

impl fmt::Display for Signal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.flag())
    }
}
