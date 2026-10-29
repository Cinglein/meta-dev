use std::panic::resume_unwind;
use std::path::Path;
use std::thread::{Scope, ScopedJoinHandle};

use super::assessed::Assessed;
use super::candidate::Candidate;
use super::census::Census;
use super::tracked::Tracked;
use super::verdict::Verdict;
use crate::request::local::local_clone::quarantine::{Quarantine, Stamp};
use crate::request::local::local_clone::vetting::{Context, Deletable, Disk, EntryKind, Occupancy, Pass, Vetted};
use crate::request::local::sweep::Sweep;

#[derive(Debug)]
pub struct Sweeper<'run> {
    sweep: Sweep,
    context: &'run Context,
}

impl<'run> Sweeper<'run> {
    pub fn new(sweep: Sweep, context: &'run Context) -> Self {
        Self { sweep, context }
    }

    pub fn context(&self) -> &Context {
        self.context
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
        match (Quarantine::holds(&path), Disk::entry(&path), tracked.lifetime().ending()) {
            (true, _, _) => stranded(&path, None),
            (false, EntryKind::Missing, _) => census
                .quarantine_of(&path)
                .map_or(Assessed::Settled(Verdict::Close { sweep, path: path.clone() }), |quarantine| {
                    stranded(quarantine, Some(path.clone()))
                }),
            (false, _, None) => Assessed::Settled(Verdict::Live { path, lifetime: tracked.lifetime() }),
            (false, _, Some(ending)) => match Vetted::proved(worktree, self.context, Pass::Swept) {
                Ok(vetted) => Assessed::Vetted(Box::new(Candidate::new(tracked, vetted, ending))),
                Err(kept) => Assessed::Settled(Verdict::Refused { sweep, path, kept, ending }),
            },
        }
    }

    pub fn assessed_together<'census>(&self, chunk: &'census [Tracked], census: &Census) -> Vec<Assessed<'census>> {
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
        'census: 'scope,
    {
        scope.spawn(move || self.assessed(tracked, census))
    }

    fn joined<'census>(run: ScopedJoinHandle<'_, Assessed<'census>>) -> Assessed<'census> {
        run.join().unwrap_or_else(|panic| resume_unwind(panic))
    }

    pub fn freed(&self, candidate: Candidate<'_>, occupancy: &Occupancy) -> Verdict {
        let sweep = self.sweep;
        let ending = candidate.ending();
        let path = candidate.vetted().link().path().to_path_buf();
        let vacant = occupancy.vacant(&path);
        let freed = match sweep {
            Sweep::Planned => vacant.map(|()| candidate.vetted().bytes()),
            Sweep::Done => vacant.and_then(|()| {
                let worktree = candidate.tracked().worktree();
                let again = Vetted::proved(worktree, self.context, Pass::Fresh);
                Deletable::confirmed(candidate.into_vetted(), again)
                    .and_then(|deletable| deletable.removed(worktree, self.context, &Stamp::current()))
            }),
        };
        match freed {
            Ok(bytes) => Verdict::Free { sweep, path, bytes, ending },
            Err(kept) => Verdict::Refused { sweep, path, kept, ending },
        }
    }
}
