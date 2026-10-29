use crate::evidence::Evidence;
use crate::promote_after::PromoteAfter;
use crate::rating_failure::RatingFailure;
use crate::reading::Reading;
use crate::scale::Scale;
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

impl<About: Subject + PartialEq, Since> Signal<About, Since> {
    pub fn triage(
        &self,
        after: PromoteAfter,
        scale: &Scale,
        earlier: &[Signal<About, Since>],
    ) -> Result<Triage, RatingFailure> {
        let promoted = after.promote(scale, self, earlier)?;
        Ok(Triage::new(self.subject.placement(), self.subject.kind(), promoted))
    }
}

#[cfg(test)]
mod tests;
