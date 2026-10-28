use super::claim_hands::ClaimHands;
use crate::request::cli_failure::CliFailure;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Undo {
    Untrailed,
    WorktreeRemoved,
    BranchDeleted,
}

impl Undo {
    pub fn applied(self, hands: &impl ClaimHands, failure: CliFailure) -> CliFailure {
        match self {
            Self::Untrailed => hands.untrailed().err().unwrap_or(failure),
            Self::WorktreeRemoved => {
                hands.removed();
                failure
            }
            Self::BranchDeleted => {
                hands.deleted();
                failure
            }
        }
    }
}
