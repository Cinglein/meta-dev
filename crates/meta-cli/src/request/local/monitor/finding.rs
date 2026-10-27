use std::fmt;

use serde::{Deserialize, Serialize};

use super::culprit::Culprit;
use super::fault::Fault;
use super::signal::Signal;
use super::watch_item::WatchItem;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Finding {
    Signalled(Signal, Culprit),
    OverLimit(Culprit),
    Unknown(Culprit),
    Watched(WatchItem),
    Blind,
    Failed(Fault),
}

impl Finding {
    pub fn repeats(&self, other: &Self) -> Option<()> {
        match (self, other) {
            (Self::OverLimit(seen), Self::OverLimit(before)) | (Self::Unknown(seen), Self::Unknown(before)) => {
                seen.repeats(before)
            }
            (Self::Watched(seen), Self::Watched(before)) => seen.same(before),
            (Self::Blind, Self::Blind) => Some(()),
            _ => None,
        }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Signalled(signal, culprit) => writeln!(formatter, "monitor sent {signal} to {culprit}"),
            Self::OverLimit(culprit) => writeln!(formatter, "monitor: agent work {culprit}"),
            Self::Unknown(culprit) => writeln!(formatter, "monitor: no agent marker under a root, {culprit}"),
            Self::Watched(item) => writeln!(formatter, "monitor: watched long enough, {item}"),
            Self::Blind => writeln!(formatter, "monitor: process table unreadable"),
            Self::Failed(fault) => writeln!(formatter, "monitor failed: {fault}"),
        }
    }
}
