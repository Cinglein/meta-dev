use std::path::PathBuf;

use serde::Deserialize;
use task::FailureKind;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(from = "String")]
enum Origin {
    Rustc,
    Lint,
}

impl From<String> for Origin {
    fn from(code: String) -> Origin {
        match code.strip_prefix('E').map(str::parse::<u16>) {
            Some(Ok(_)) => Origin::Rustc,
            _ => Origin::Lint,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct Code {
    code: Origin,
}

impl Code {
    pub(super) fn failure(&self, paths: Vec<PathBuf>) -> FailureKind {
        match self.code {
            Origin::Rustc => FailureKind::Compile { paths },
            Origin::Lint => FailureKind::Lint { paths },
        }
    }
}
