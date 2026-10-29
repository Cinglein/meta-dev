use super::super::culprit::Culprit;
use super::super::fault::Fault;
use super::super::finding::Finding;
use super::super::reused::Reused;

#[derive(Debug, Default)]
pub struct Confirmation {
    kept: Vec<Culprit>,
    refused: Vec<Finding>,
}

impl Confirmation {
    #[must_use]
    pub fn judged(self, culprit: Culprit, outcome: Result<(), Reused>) -> Self {
        match outcome {
            Ok(()) => Self { kept: self.kept.into_iter().chain([culprit]).collect(), ..self },
            Err(reused) => {
                let refusal = Finding::Failed(Fault::shown(reused));
                Self { refused: self.refused.into_iter().chain([refusal]).collect(), ..self }
            }
        }
    }

    pub fn kept(&self) -> &[Culprit] {
        &self.kept
    }

    pub fn refused(&self) -> &[Finding] {
        &self.refused
    }
}
