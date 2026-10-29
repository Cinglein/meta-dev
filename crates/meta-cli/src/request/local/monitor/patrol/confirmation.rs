mod reused;

use super::super::culprit::Culprit;
use super::super::fault::Fault;
use super::super::finding::Finding;
use super::super::standing::Standing;
use reused::Reused;

#[derive(Debug, Default)]
pub struct Confirmation {
    kept: Vec<Culprit>,
    refused: Vec<Finding>,
}

impl Confirmation {
    #[must_use]
    pub fn judged(self, culprit: Culprit, standing: Standing) -> Self {
        match standing {
            Standing::Same => Self { kept: self.kept.into_iter().chain([culprit]).collect(), ..self },
            Standing::Gone => {
                Self { refused: self.refused.into_iter().chain([Finding::Gone(culprit)]).collect(), ..self }
            }
            Standing::Changed => {
                let refusal = Finding::Failed(Fault::shown(Reused::new(culprit.pid())));
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
