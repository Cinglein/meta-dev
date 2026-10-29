mod assessed;
mod candidate;
mod census;
mod surveyed;
mod sweeper;
mod tracked;
mod verdict;

use std::fmt;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};

use crate::request::local::local_clone::vetting::Occupancy;
use crate::request::local::sweep::Sweep;
use crate::request::local::trail_entry::TrailEntry;
use assessed::Assessed;
pub use census::Census;
pub use surveyed::Surveyed;
pub use sweeper::Sweeper;
pub use tracked::Tracked;
use verdict::Verdict;

#[derive(Debug)]
pub struct Collection(Vec<Verdict>);

impl Collection {
    pub fn swept(sweeper: &Sweeper<'_>, census: &Census, surveyed: &Surveyed) -> Self {
        let sorted = sweeper.sorted(census, surveyed);
        let vetting: Vec<_> = sorted.iter().filter_map(|sorted| sorted.as_ref().err().copied()).collect();
        let mut vetted = sweeper.vetted(&vetting, surveyed).into_iter();
        let assessed: Vec<Assessed<'_>> =
            sorted.into_iter().filter_map(|sorted| sorted.map_or_else(|_| vetted.next(), Some)).collect();
        let candidates: Vec<PathBuf> = assessed.iter().filter_map(Assessed::candidate).map(Path::to_path_buf).collect();
        let occupancy = match sweeper.sweep() {
            Sweep::Planned if !candidates.is_empty() => sweeper.scanned(&candidates),
            Sweep::Planned | Sweep::Done => Occupancy::Listed(Vec::new()),
        };
        let finished = assessed.into_iter().map(|assessed| assessed.verdict(|candidate| sweeper.freed(candidate, &occupancy)));
        let unrecorded = census.unrecorded().map(|path| sweeper.unrecorded(path));
        let mut verdicts: Vec<Verdict> = finished.chain(unrecorded).collect();
        verdicts.sort_by_key(|verdict| verdict.stranded().is_none());
        Self(verdicts)
    }

    pub fn trailed(&self, at: DateTime<Utc>) -> Vec<TrailEntry> {
        self.0.iter().filter_map(|verdict| verdict.trailed(at)).collect()
    }
}

impl fmt::Display for Collection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|verdict| writeln!(formatter, "{verdict}"))
    }
}

#[cfg(test)]
mod tests;
