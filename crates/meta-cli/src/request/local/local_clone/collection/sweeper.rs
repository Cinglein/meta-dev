use std::path::{Path, PathBuf};

use meta_status::Ending;

use super::assessed::Assessed;
use super::candidate::Candidate;
use super::census::Census;
use super::surveyed::Surveyed;
use super::tracked::Tracked;
use super::verdict::Verdict;
use crate::request::local::local_clone::quarantine::{Quarantine, Stamp};
use crate::request::local::local_clone::vetting::{
    Cause, Context, Deletable, EntryKind, Halt, Interpreter, Kept, Occupancy, Replayed, Vetted,
};
use crate::request::local::sweep::Sweep;

#[derive(Debug)]
pub struct Sweeper<'run> {
    sweep: Sweep,
    context: Context,
    interpreter: &'run Interpreter<'run>,
    stamp: Stamp,
}

impl<'run> Sweeper<'run> {
    pub fn new(sweep: Sweep, context: Context, interpreter: &'run Interpreter<'run>, stamp: Stamp) -> Self {
        Self { sweep, context, interpreter, stamp }
    }

    pub fn surveyed(&self, census: &Census) -> Result<Surveyed, Kept> {
        let paths: Vec<PathBuf> = census.tracked().iter().map(|tracked| tracked.worktree().path().to_path_buf()).collect();
        let vetting = census.tracked().iter().any(|tracked| tracked.lifetime().ending().is_some());
        let context = &self.context;
        self.interpreter.ran(&Replayed::new(|replay| Surveyed::read(context, &paths, vetting, replay)))?
    }

    pub fn lent(self, surveyed: &Surveyed) -> Self {
        Self { context: self.context.lent(surveyed.lenders()), ..self }
    }

    pub fn sweep(&self) -> Sweep {
        self.sweep
    }

    pub fn unrecorded(&self, path: &Path) -> Verdict {
        let path = path.to_path_buf();
        if Quarantine::holds(&path) {
            Verdict::Stranded { sweep: self.sweep, path, record: None }
        } else {
            Verdict::Unrecorded { path }
        }
    }

    // In census order: a verdict reached without vetting, or the worktree to vet with its ending.
    pub fn sorted<'census>(
        &self,
        census: &'census Census,
        surveyed: &Surveyed,
    ) -> Vec<Result<Assessed<'census>, (&'census Tracked, Ending)>> {
        let sweep = self.sweep;
        let stranded =
            |path: &Path, record| Ok(Assessed::Settled(Verdict::Stranded { sweep, path: path.to_path_buf(), record }));
        let sort = |(at, tracked): (usize, &'census Tracked)| {
            let path = tracked.worktree().path().to_path_buf();
            match (Quarantine::holds(&path), surveyed.entry(at), tracked.lifetime().ending()) {
                (true, _, _) => stranded(&path, None),
                (false, EntryKind::Missing, _) => census.quarantine_of(&path).map_or(
                    Ok(Assessed::Settled(Verdict::Close { sweep, path: path.clone() })),
                    |quarantine| stranded(quarantine, Some(path.clone())),
                ),
                (false, _, None) => Ok(Assessed::Settled(Verdict::Live { path, lifetime: tracked.lifetime() })),
                (false, _, Some(ending)) => Err((tracked, ending)),
            }
        };
        census.tracked().iter().enumerate().map(sort).collect()
    }

    // All vets run in lockstep: each round's reads from every worktree go out as one batch.
    pub fn vetted<'census>(&self, vetting: &[(&'census Tracked, Ending)], surveyed: &Surveyed) -> Vec<Assessed<'census>> {
        let context = &self.context;
        let decisions: Vec<_> = vetting
            .iter()
            .map(|(tracked, _)| {
                Replayed::new(move |replay| {
                    Vetted::proved_among(tracked.worktree(), context, replay, surveyed.checkouts(), surveyed.remote())
                })
            })
            .collect();
        let judged = self.interpreter.ran_all(&decisions);
        vetting
            .iter()
            .zip(judged)
            .map(|((tracked, ending), judged)| match judged.and_then(|judged| judged) {
                Ok(vetted) => Assessed::Vetted(Box::new(Candidate::new(tracked, vetted, *ending))),
                Err(kept) => Assessed::Settled(Verdict::Refused {
                    sweep: self.sweep,
                    path: tracked.worktree().path().to_path_buf(),
                    kept,
                    ending: *ending,
                }),
            })
            .collect()
    }

    pub fn scanned(&self, candidates: &[PathBuf]) -> Occupancy {
        let scan = self.interpreter.ran(&Replayed::new(|replay| replay.occupancy(candidates)));
        scan.and_then(|scan| scan).unwrap_or_else(|kept| Occupancy::Unlisted(Cause::new(kept)))
    }

    pub fn freed(&self, candidate: Candidate<'_>, occupancy: &Occupancy) -> Verdict {
        let sweep = self.sweep;
        let ending = candidate.ending();
        let path = candidate.vetted().link().path().to_path_buf();
        let freed = match sweep {
            Sweep::Planned => occupancy.vacant(&path).map(|()| candidate.vetted().bytes()),
            Sweep::Done => {
                let worktree = candidate.tracked().worktree();
                let first = candidate.into_vetted();
                let (context, stamp) = (&self.context, &self.stamp);
                let removal = Replayed::new(|replay| {
                    let again = Halt::settled(Vetted::proved(worktree, context, replay))?;
                    Deletable::confirmed(first.clone(), again)?.removed(worktree, context, stamp, replay)
                });
                self.interpreter.ran(&removal).and_then(|removed| removed)
            }
        };
        match freed {
            Ok(bytes) => Verdict::Free { sweep, path, bytes, ending },
            Err(kept) => Verdict::Refused { sweep, path, kept, ending },
        }
    }
}
