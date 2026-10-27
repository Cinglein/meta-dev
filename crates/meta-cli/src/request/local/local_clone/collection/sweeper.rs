use std::cell::LazyCell;
use std::panic::resume_unwind;
use std::path::Path;
use std::thread::{Scope, ScopedJoinHandle};

use super::assessed::Assessed;
use super::candidate::Candidate;
use super::census::Census;
use super::tracked::Tracked;
use super::verdict::Verdict;
use crate::request::local::local_clone::quarantine::Quarantine;
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

    pub fn unrecorded(&self, path: &Path) -> Verdict {
        let path = path.to_path_buf();
        if Quarantine::holds(&path) {
            Verdict::Stranded { sweep: self.sweep, path, record: None }
        } else {
            Verdict::Unrecorded { path }
        }
    }

    pub fn assessed<'census>(&self, tracked: &'census Tracked, census: &Census) -> Assessed<'census> {
        let sweep = self.sweep;
        let worktree = tracked.worktree();
        let path = worktree.path().to_path_buf();
        let stranded =
            |path: &Path, record| Assessed::Settled(Verdict::Stranded { sweep, path: path.to_path_buf(), record });
        match (Quarantine::holds(&path), self.host.entry(&path), tracked.lifetime().ending()) {
            (true, _, _) => stranded(&path, None),
            (false, EntryKind::Missing, _) => census
                .quarantine_of(&path)
                .map_or(Assessed::Settled(Verdict::Close { sweep, path: path.clone() }), |quarantine| {
                    stranded(quarantine, Some(path.clone()))
                }),
            (false, _, None) => Assessed::Settled(Verdict::Live { path, lifetime: tracked.lifetime() }),
            (false, _, Some(ending)) => match Vetted::proved(worktree, self.context, self.host) {
                Ok(vetted) => Assessed::Vetted(Box::new(Candidate::new(tracked, vetted, ending))),
                Err(kept) => Assessed::Settled(Verdict::Refused { sweep, path, kept, ending }),
            },
        }
    }

    pub fn assessed_together<'census>(&self, chunk: &'census [Tracked], census: &Census) -> Vec<Assessed<'census>>
    where
        Host: Sync,
    {
        std::thread::scope(|scope| {
            let running: Vec<_> = chunk.iter().map(|tracked| self.started(scope, tracked, census)).collect();
            running.into_iter().map(Self::joined).collect()
        })
    }

    fn started<'scope, 'census>(
        &'scope self,
        scope: &'scope Scope<'scope, '_>,
        tracked: &'census Tracked,
        census: &'scope Census,
    ) -> ScopedJoinHandle<'scope, Assessed<'census>>
    where
        Host: Sync,
        'census: 'scope,
    {
        scope.spawn(move || self.assessed(tracked, census))
    }

    fn joined<'census>(run: ScopedJoinHandle<'_, Assessed<'census>>) -> Assessed<'census> {
        run.join().unwrap_or_else(|panic| resume_unwind(panic))
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
                let worktree = candidate.tracked().worktree();
                let again = Vetted::proved(worktree, self.context, self.host);
                Deletable::confirmed(candidate.into_vetted(), again)
                    .and_then(|deletable| deletable.removed(worktree, self.context, self.host))
            }
        };
        match freed {
            Ok(bytes) => Verdict::Free { sweep, path, bytes, ending },
            Err(kept) => Verdict::Refused { sweep, path, kept, ending },
        }
    }
}
