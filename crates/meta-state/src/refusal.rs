use meta_schema::{Actor, Division, Entry, Id, Project, PullRequest, Rating, SchemaVersion, Target, Task};
use thiserror::Error;

use crate::project_stage::ProjectStage;

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum Refusal {
    #[error("the id is already in use")]
    DuplicateId,
    #[error("there is no project {0}")]
    UnknownProject(Id<Project>),
    #[error("only the owner accepts a project")]
    NotOwner,
    #[error("the project is {0}")]
    ProjectIs(ProjectStage),
    #[error("the observation does not change the stage")]
    Unchanged,
    #[error("a task with this purpose cannot attach to a {0} project")]
    Unattachable(ProjectStage),
    #[error("the task already has a purpose")]
    AlreadyPurposed,
    #[error("there is no task {0}")]
    UnknownTask(Id<Task>),
    #[error("there is no concern or issue {0}")]
    UnknownTarget(Target),
    #[error("the task is already done")]
    AlreadyDone,
    #[error("the task is not done")]
    NotDone,
    #[error("{0} is already assigned to the task")]
    AlreadyAssigned(Actor),
    #[error("{0} is not assigned to the task")]
    NotAssigned(Actor),
    #[error("meta/log was written by a newer meta (schema {0}); upgrade meta")]
    NewerSchema(SchemaVersion),
    #[error("{0} is closed")]
    Closed(Entry),
    #[error("waits would form a loop")]
    WaitCycle,
    #[error("the task already waits; resume it first")]
    AlreadyWaiting,
    #[error("the last holder cannot leave a task that waits; resume it or finish it")]
    WaitWithoutHolder,
    #[error("the task is not waiting")]
    NotBlocked,
    #[error("a task cannot wait on itself")]
    WaitsOnItself,
    #[error("{0} is already linked to the task")]
    AlreadyLinked(PullRequest),
    #[error("there is no entry {0}")]
    UnknownEntry(Entry),
    #[error("{0} is retired")]
    Retired(Entry),
    #[error("the item is {0} now; rate it again")]
    RatedSince(Rating),
    #[error("the item has moved since; move it again")]
    MovedSince,
    #[error("the item is already in {0}")]
    AlreadyIn(Division),
}
