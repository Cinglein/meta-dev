use std::cell::Cell;
use std::path::{Path, PathBuf};

use super::alternates::Alternates;
use super::cause::Cause;
use super::change::Change;
use super::entry_kind::EntryKind;
use super::found::Found;
use super::git_call::GitCall;
use super::halt::Halt;
use super::kept::Kept;
use super::look::Look;
use super::made::Made;
use super::occupancy::Occupancy;
use super::seen::Seen;
use super::thaw::Thaw;
use super::turn::Turn;

// A decision reads its journal through a Replay. Each read takes the next turn, in order; past the
// end it halts with the read it needs. Nothing is cached: reading the same thing twice is two turns,
// so a re-read is always a new answer from the world, taken after every earlier turn.
#[derive(Debug)]
pub struct Replay<'journal> {
    journal: &'journal [Turn],
    at: Cell<usize>,
}

impl<'journal> Replay<'journal> {
    const DIVERGED: &'static str = "the journal does not match the decision";

    pub fn new(journal: &'journal [Turn]) -> Self {
        Self { journal, at: Cell::new(0) }
    }

    pub fn diverged() -> Halt {
        Halt::Kept(Kept::Uninspected(Cause::new(Self::DIVERGED)))
    }

    pub fn looked(&self, looks: Vec<Look>) -> Result<Vec<Seen>, Halt> {
        let at = self.at.get();
        match self.journal.get(at) {
            None => Err(Halt::Look(looks)),
            Some(Turn::Looked(asked, seen)) if *asked == looks && seen.len() == looks.len() => {
                self.at.set(at + 1);
                Ok(seen.clone())
            }
            Some(_) => Err(Self::diverged()),
        }
    }

    pub fn acted(&self, changes: Vec<Change>) -> Result<Vec<Made>, Halt> {
        let at = self.at.get();
        match self.journal.get(at) {
            None => Err(Halt::Act(changes)),
            Some(Turn::Acted(asked, made)) if *asked == changes && made.len() == changes.len() => {
                self.at.set(at + 1);
                Ok(made.clone())
            }
            Some(_) => Err(Self::diverged()),
        }
    }

    pub fn entry(&self, path: &Path) -> Result<EntryKind, Halt> {
        self.one(Look::Entry(path.to_path_buf()))?.entry()
    }

    pub fn canonical(&self, path: &Path) -> Result<Option<PathBuf>, Halt> {
        self.one(Look::Canonical(path.to_path_buf()))?.canonical()
    }

    pub fn text(&self, path: &Path) -> Result<Option<String>, Halt> {
        self.one(Look::Text(path.to_path_buf()))?.text()
    }

    pub fn git(&self, call: GitCall) -> Result<Result<String, Kept>, Halt> {
        self.one(Look::Git(call))?.git()
    }

    pub fn occupancy(&self, paths: &[PathBuf]) -> Result<Occupancy, Halt> {
        self.one(Look::Occupancy(paths.to_vec()))?.occupancy()
    }

    pub fn changed(&self, call: GitCall) -> Result<Result<String, Kept>, Halt> {
        self.made(Change::Git(call))?.git()
    }

    pub fn destroyed(&self, call: GitCall) -> Result<Result<String, Kept>, Halt> {
        self.made(Change::Destroy(call))?.git()
    }

    pub fn frozen(&self, quarantine: &Path, plan: Thaw) -> Result<Result<Thaw, Kept>, Halt> {
        match self.made(Change::Freeze(quarantine.to_path_buf(), plan))? {
            Made::Frozen(frozen) => Ok(frozen),
            Made::Git(_) | Made::Thawed => Err(Self::diverged()),
        }
    }

    pub fn thawed(&self, thaw: &Thaw) -> Result<(), Halt> {
        self.made(Change::Thaw(thaw.clone())).map(drop)
    }

    fn one(&self, look: Look) -> Result<Seen, Halt> {
        self.looked(vec![look])?.into_iter().next().ok_or_else(Self::diverged)
    }

    fn made(&self, change: Change) -> Result<Made, Halt> {
        self.acted(vec![change])?.into_iter().next().ok_or_else(Self::diverged)
    }
}

impl Seen {
    pub fn entry(self) -> Result<EntryKind, Halt> {
        if let Self::Entry(kind) = self { Ok(kind) } else { Err(Replay::diverged()) }
    }

    pub fn canonical(self) -> Result<Option<PathBuf>, Halt> {
        if let Self::Canonical(path) = self { Ok(path) } else { Err(Replay::diverged()) }
    }

    pub fn text(self) -> Result<Option<String>, Halt> {
        if let Self::Text(text) = self { Ok(text) } else { Err(Replay::diverged()) }
    }

    pub fn git(self) -> Result<Result<String, Kept>, Halt> {
        if let Self::Git(output) = self { Ok(output) } else { Err(Replay::diverged()) }
    }

    pub fn walked(self) -> Result<Vec<Found>, Halt> {
        if let Self::Walk(found) = self { Ok(found) } else { Err(Replay::diverged()) }
    }

    pub fn occupancy(self) -> Result<Occupancy, Halt> {
        if let Self::Occupancy(occupancy) = self { Ok(occupancy) } else { Err(Replay::diverged()) }
    }

    pub fn lenders(self) -> Result<Vec<Alternates>, Halt> {
        if let Self::Lenders(lenders) = self { Ok(lenders) } else { Err(Replay::diverged()) }
    }
}

impl Made {
    pub fn git(self) -> Result<Result<String, Kept>, Halt> {
        if let Self::Git(output) = self { Ok(output) } else { Err(Replay::diverged()) }
    }
}
