use std::fmt;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::super::fault::Fault;
use crate::request::hook::CallPhase;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Hush {
    Quiet { quiet: Duration, limit: Duration },
    LongCall { quiet: Duration, limit: Duration },
    Unstamped,
    Unreadable(Fault),
}

impl Hush {
    pub fn timed(call: CallPhase, quiet: Duration, limit: Duration) -> Option<Self> {
        let quiet = Duration::from_secs(quiet.as_secs());
        (quiet > limit).then_some(match call {
            CallPhase::Started => Self::LongCall { quiet, limit },
            CallPhase::Finished => Self::Quiet { quiet, limit },
        })
    }

    pub fn stops(&self) -> Option<()> {
        matches!(self, Self::Quiet { .. }).then_some(())
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Quiet { .. } => "silent",
            Self::LongCall { .. } => "long call",
            Self::Unstamped | Self::Unreadable(_) => "unstamped",
        }
    }
}

impl fmt::Display for Hush {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Quiet { quiet, limit } => {
                write!(formatter, "no tool call for {}s, limit {}s", quiet.as_secs(), limit.as_secs())
            }
            Self::LongCall { quiet, limit } => {
                write!(formatter, "one tool call running {}s, limit {}s, report only", quiet.as_secs(), limit.as_secs())
            }
            Self::Unstamped => formatter.write_str("live worktree with no tool-call stamp"),
            Self::Unreadable(fault) => write!(formatter, "tool-call stamp unreadable: {fault}"),
        }
    }
}
