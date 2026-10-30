use meta_schema::{Id, Task};
use meta_state::TaskStatus;
use thiserror::Error;

use super::Rules;

#[derive(Debug, Error)]
pub enum BriefFailure {
    #[error("unknown task {0}")]
    UnknownTask(Id<Task>),
    #[error("task {0} is {1}; a brief needs a live task")]
    Closed(Id<Task>, TaskStatus),
    #[error("{file}: {0}", file = Rules::FILE)]
    Unreadable(std::io::Error),
    #[error("{file}: first line must be `version: <number>`", file = Rules::FILE)]
    Unversioned,
}
