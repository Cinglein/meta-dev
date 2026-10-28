mod branch_refs;
mod claim_hands;
mod claim_refusal;
mod claim_step;
mod holding;
mod undo;

use std::path::Path;

pub use branch_refs::BranchRefs;
pub use claim_hands::ClaimHands;
pub use claim_refusal::ClaimRefusal;
pub use claim_step::ClaimStep;
pub use holding::Holding;
pub use undo::Undo;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClaimPlan(Vec<ClaimStep>);

impl ClaimPlan {
    pub fn new(holding: Holding, refs: &BranchRefs, taken: Option<&Path>) -> Result<Self, ClaimRefusal> {
        let base = refs.base(holding)?;
        taken.map_or(Ok(()), |path| Err(ClaimRefusal::Taken(path.to_path_buf())))?;
        let written = (holding == Holding::Fresh).then_some(ClaimStep::Written);
        let steps = [ClaimStep::Pruned, ClaimStep::Configured].into_iter().chain(base.map(ClaimStep::Branched));
        Ok(Self(steps.chain([ClaimStep::Added, ClaimStep::Settled]).chain(written).collect()))
    }

    pub fn steps(&self) -> &[ClaimStep] {
        &self.0
    }

    pub fn rollback(done: &[ClaimStep]) -> Vec<Undo> {
        done.iter().rev().filter_map(ClaimStep::undo).collect()
    }
}

#[cfg(test)]
mod tests;
