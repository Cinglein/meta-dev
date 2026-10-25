use meta_schema::{Actor, Id, Target, Task};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum Refusal {
    #[error("the id is already in use")]
    DuplicateId,
    #[error("there is no task {0}")]
    UnknownTask(Id<Task>),
    #[error("there is no concern or issue {0}")]
    UnknownTarget(Target),
    #[error("the task is already done")]
    AlreadyDone,
    #[error("the task is not done")]
    NotDone,
    #[error("the task is claimed by {0}; take it over with --from {0}")]
    ClaimedBy(Actor),
    #[error("{0} does not hold the task")]
    NotClaimant(Actor),
    #[error("the task is held by {0}, not by you")]
    HeldBy(Actor),
    #[error("the task is not claimed; claim it first")]
    Unclaimed,
}
