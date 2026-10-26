use std::cell::LazyCell;

use super::assessed::Assessed;
use super::candidate::Candidate;
use super::tracked::Tracked;
use super::verdict::Verdict;
use crate::request::local::local_clone::vetting::{Context, Deletable, EntryKind, Inspection, Occupancy, Vetted};
use crate::request::local::sweep::Sweep;

#[derive(Debug)]
pub struct Sweeper<'run, Host: Inspection> {
    sweep: Sweep,
    context: &'run Context,
    host: &'run Host,
}

impl<'run, Host: Inspection> Sweeper<'run, Host> {
    pub fn new(sweep: Sweep, context: &'run Context, host: &'run Host) -> Self {
        Self { sweep, context, host }
    }

    pub fn host(&self) -> &Host {
        self.host
    }

    pub fn assessed<'census>(&self, tracked: &'census Tracked) -> Assessed<'census> {
        let sweep = self.sweep;
        let worktree = tracked.worktree();
        let path = worktree.path().to_path_buf();
        match (self.host.entry(&path), tracked.lifetime().ending()) {
            (EntryKind::Missing, _) => Assessed::Settled(Verdict::Close { sweep, path }),
            (_, None) => Assessed::Settled(Verdict::Live { path, lifetime: tracked.lifetime() }),
            (_, Some(ending)) => match Vetted::proved(worktree, self.context, self.host) {
                Ok(vetted) => Assessed::Vetted(Box::new(Candidate::new(tracked, vetted, ending))),
                Err(kept) => Assessed::Settled(Verdict::Refused { sweep, path, kept, ending }),
            },
        }
    }

    pub fn freed(
        &self,
        candidate: Candidate<'_>,
        occupancy: &LazyCell<Occupancy, impl FnOnce() -> Occupancy>,
    ) -> Verdict {
        let sweep = self.sweep;
        let ending = candidate.ending();
        let path = candidate.vetted().link().path().to_path_buf();
        let freed = match sweep {
            Sweep::Planned => occupancy.vacant(&path).map(|()| candidate.vetted().bytes()),
            Sweep::Done => {
                let again = Vetted::proved(candidate.tracked().worktree(), self.context, self.host);
                Deletable::confirmed(candidate.into_vetted(), occupancy, again)
                    .and_then(|deletable| deletable.removed(self.host))
            }
        };
        match freed {
            Ok(bytes) => Verdict::Free { sweep, path, bytes, ending },
            Err(kept) => Verdict::Refused { sweep, path, kept, ending },
        }
    }
}
