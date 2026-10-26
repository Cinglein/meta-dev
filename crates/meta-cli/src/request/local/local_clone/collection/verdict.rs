use std::fmt;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use meta_status::{Ending, Lifetime};

use crate::request::local::byte_count::ByteCount;
use crate::request::local::local_clone::vetting::Kept;
use crate::request::local::reason::Reason;
use crate::request::local::sweep::Sweep;
use crate::request::local::trail_entry::TrailEntry;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    Free { sweep: Sweep, path: PathBuf, bytes: ByteCount, ending: Ending },
    Close { sweep: Sweep, path: PathBuf },
    Live { path: PathBuf, lifetime: Lifetime },
    Refused { sweep: Sweep, path: PathBuf, kept: Kept, ending: Ending },
    Unrecorded { path: PathBuf },
    Stranded { sweep: Sweep, path: PathBuf, record: Option<PathBuf> },
}

impl Verdict {
    pub fn stranded(&self) -> Option<()> {
        match self {
            Self::Stranded { .. } => Some(()),
            Self::Free { .. }
            | Self::Close { .. }
            | Self::Live { .. }
            | Self::Refused { .. }
            | Self::Unrecorded { .. } => None,
        }
    }

    pub fn trailed(&self, at: DateTime<Utc>) -> Option<TrailEntry> {
        match self {
            Self::Free { sweep: Sweep::Done, path, bytes, ending } => {
                let reason: Reason = ending.to_string().parse().unwrap_or_else(|never| match never {});
                Some(TrailEntry::Freed { path: path.clone(), at, reason, bytes: *bytes })
            }
            Self::Close { sweep: Sweep::Done, path } => Some(TrailEntry::Closed { path: path.clone(), at }),
            Self::Refused { sweep: Sweep::Done, path, kept, ending } => {
                let reason: Reason = format!("{kept}; {ending}").parse().unwrap_or_else(|never| match never {});
                Some(TrailEntry::Refused { path: path.clone(), at, reason })
            }
            Self::Stranded { sweep: Sweep::Done, path, record: Some(record) } => {
                Some(TrailEntry::Moved { path: record.clone(), to: path.clone(), at })
            }
            Self::Free { .. }
            | Self::Close { .. }
            | Self::Refused { .. }
            | Self::Live { .. }
            | Self::Unrecorded { .. }
            | Self::Stranded { .. } => None,
        }
    }
}

impl fmt::Display for Verdict {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Free { sweep: Sweep::Planned, path, bytes, ending } => {
                write!(formatter, "would free {}: {bytes}; {ending}", path.display())
            }
            Self::Free { sweep: Sweep::Done, path, bytes, ending } => {
                write!(formatter, "freed {}: {bytes}; {ending}", path.display())
            }
            Self::Close { sweep: Sweep::Planned, path } => write!(formatter, "would close {}: missing", path.display()),
            Self::Close { sweep: Sweep::Done, path } => write!(formatter, "closed {}: missing", path.display()),
            Self::Live { path, lifetime } => write!(formatter, "kept {}: {lifetime}", path.display()),
            Self::Refused { path, kept, ending, .. } => write!(formatter, "kept {}: {kept}; {ending}", path.display()),
            Self::Unrecorded { path } => write!(formatter, "unrecorded {}: left alone", path.display()),
            Self::Stranded { path, record, .. } => {
                write!(formatter, "STRANDED in quarantine {}: left alone, move it back by hand", path.display())?;
                record.as_ref().map_or(Ok(()), |record| write!(formatter, "; record follows from {}", record.display()))
            }
        }
    }
}
