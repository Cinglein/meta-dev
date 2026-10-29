use meta_schema::{ItemKind, LevelName, Placement};

use crate::evidence::Evidence;
use crate::subject::Subject;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signal<About, Since> {
    subject: About,
    level: Option<LevelName>,
    evidence: Evidence,
    since: Since,
}

impl<About, Since> Signal<About, Since> {
    #[must_use]
    pub fn new(subject: About, level: Option<LevelName>, evidence: Evidence, since: Since) -> Signal<About, Since> {
        Signal { subject, level, evidence, since }
    }

    #[must_use]
    pub fn identity(&self) -> &About {
        &self.subject
    }

    #[must_use]
    pub fn level(&self) -> Option<&LevelName> {
        self.level.as_ref()
    }

    #[must_use]
    pub fn evidence(&self) -> &Evidence {
        &self.evidence
    }

    #[must_use]
    pub fn since(&self) -> &Since {
        &self.since
    }
}

impl<About: Subject, Since> Signal<About, Since> {
    #[must_use]
    pub fn triage(&self) -> (Placement, ItemKind, Option<&LevelName>) {
        (self.subject.placement(), self.subject.kind(), self.level())
    }
}

#[cfg(test)]
mod tests;
