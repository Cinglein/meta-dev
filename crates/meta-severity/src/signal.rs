use crate::evidence::Evidence;
use crate::reading::Reading;
use crate::subject::Subject;
use crate::triage::Triage;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signal<About, Since> {
    subject: About,
    reading: Reading,
    evidence: Evidence,
    since: Since,
}

impl<About, Since> Signal<About, Since> {
    #[must_use]
    pub fn new(subject: About, reading: Reading, evidence: Evidence, since: Since) -> Signal<About, Since> {
        Signal { subject, reading, evidence, since }
    }

    #[must_use]
    pub fn identity(&self) -> &About {
        &self.subject
    }

    #[must_use]
    pub fn reading(&self) -> &Reading {
        &self.reading
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
    pub fn triage(&self) -> Triage {
        Triage::new(self.subject.placement(), self.subject.kind(), self.reading.clone())
    }
}

#[cfg(test)]
mod tests;
