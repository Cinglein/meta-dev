use super::claim_hands::ClaimHands;
use super::undo::Undo;
use crate::request::cli_failure::CliFailure;
use crate::request::project::BranchName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClaimStep {
    Pruned,
    Configured,
    Branched(BranchName),
    Added,
    Settled,
    Written,
}

impl ClaimStep {
    pub fn undo(&self) -> Option<Undo> {
        match self {
            Self::Branched(_) => Some(Undo::BranchDeleted),
            Self::Added => Some(Undo::WorktreeRemoved),
            Self::Settled => Some(Undo::Untrailed),
            Self::Pruned | Self::Configured | Self::Written => None,
        }
    }

    pub fn taken(&self, hands: &impl ClaimHands) -> Result<String, CliFailure> {
        let quiet = |acted: Result<(), CliFailure>| acted.map(|()| String::new());
        match self {
            Self::Pruned => quiet(hands.pruned()),
            Self::Configured => quiet(hands.configured()),
            Self::Branched(base) => quiet(hands.branched(base)),
            Self::Added => quiet(hands.added()),
            Self::Settled => quiet(hands.settled()),
            Self::Written => hands.written(),
        }
    }
}
