mod assessed;
mod candidate;
mod census;
mod sweeper;
mod tracked;
mod verdict;

use std::cell::LazyCell;
use std::fmt;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};

use crate::request::local::local_clone::vetting::Inspection;
use crate::request::local::trail_entry::TrailEntry;
use assessed::Assessed;
pub use census::Census;
use meta_process_table::Uninspected;
pub use sweeper::Sweeper;
pub use tracked::Tracked;
use verdict::Verdict;

#[derive(Debug)]
pub struct Collection(Vec<Verdict>, Uninspected);

impl Collection {
    pub fn swept<Host: Inspection + Sync>(sweeper: &Sweeper<'_, Host>, census: &Census) -> Self {
        let width = std::thread::available_parallelism().map_or(1, NonZeroUsize::get);
        let assessed: Vec<Assessed<'_>> =
            census.tracked().chunks(width).flat_map(|chunk| sweeper.assessed_together(chunk, census)).collect();
        let candidates: Vec<PathBuf> = assessed.iter().filter_map(Assessed::candidate).map(Path::to_path_buf).collect();
        let occupancy = LazyCell::new(|| sweeper.host().occupancy(&candidates));
        let finished =
            assessed.into_iter().map(|assessed| assessed.verdict(|candidate| sweeper.freed(candidate, &occupancy)));
        let unrecorded = census.unrecorded().map(|path| sweeper.unrecorded(path));
        let mut verdicts: Vec<Verdict> = finished.chain(unrecorded).collect();
        verdicts.sort_by_key(|verdict| verdict.stranded().is_none());
        Self(verdicts, sweeper.host().uninspected())
    }

    pub fn trailed(&self, at: DateTime<Utc>) -> Vec<TrailEntry> {
        self.0.iter().filter_map(|verdict| verdict.trailed(at)).collect()
    }
}

impl fmt::Display for Collection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|verdict| writeln!(formatter, "{verdict}"))?;
        write!(formatter, "{}", self.1)
    }
}

#[cfg(test)]
mod tests;
