use std::fmt;

use serde::{Deserialize, Serialize};

use super::culprit::Culprit;
use super::signal::Signal;
use crate::request::local::local_settings::CoreShare;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Finding {
    Signalled(Signal, Culprit),
    Unsure(Culprit),
    Load { seen: CoreShare, limit: CoreShare },
    AgentCpu { seen: CoreShare, limit: CoreShare },
    Blind,
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Signalled(signal, culprit) => writeln!(formatter, "monitor sent {signal} to {culprit}"),
            Self::Unsure(culprit) => writeln!(formatter, "monitor left alone, unsure it is agent work: {culprit}"),
            Self::Load { seen, limit } => writeln!(formatter, "monitor: load {seen} cores, limit {limit}"),
            Self::AgentCpu { seen, limit } => {
                writeln!(formatter, "monitor: agent processes use {seen} cores, limit {limit}")
            }
            Self::Blind => writeln!(formatter, "monitor: process table unreadable, nothing stopped"),
        }
    }
}
