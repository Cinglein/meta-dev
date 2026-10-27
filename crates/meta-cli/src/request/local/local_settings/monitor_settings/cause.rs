use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Cause {
    Orphaned,
    CpuTime,
    WallTime,
}

impl fmt::Display for Cause {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Orphaned => "orphaned past its limit",
            Self::CpuTime => "past its cpu time limit",
            Self::WallTime => "past its wall time limit",
        })
    }
}
