use std::path::PathBuf;

use super::alternates::Alternates;
use super::entry_kind::EntryKind;
use super::found::Found;
use super::kept::Kept;
use super::occupancy::Occupancy;

// The view one Look returned, parsed where the parser is the view's own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Seen {
    Entry(EntryKind),
    Canonical(Option<PathBuf>),
    Text(Option<String>),
    Git(Result<String, Kept>),
    Walk(Vec<Found>),
    Lenders(Vec<Alternates>),
    Occupancy(Occupancy),
}
