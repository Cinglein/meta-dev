use std::path::Path;

use super::candidate::Candidate;
use super::verdict::Verdict;

#[derive(Debug)]
pub enum Assessed<'census> {
    Settled(Verdict),
    Vetted(Box<Candidate<'census>>),
}

impl<'census> Assessed<'census> {
    pub fn candidate(&self) -> Option<&Path> {
        match self {
            Self::Vetted(candidate) => Some(candidate.vetted().link().path()),
            Self::Settled(_) => None,
        }
    }

    pub fn verdict(self, freed: impl FnOnce(Candidate<'census>) -> Verdict) -> Verdict {
        match self {
            Self::Settled(verdict) => verdict,
            Self::Vetted(candidate) => freed(*candidate),
        }
    }
}
