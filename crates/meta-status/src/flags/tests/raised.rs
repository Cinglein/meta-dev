use meta_schema::{DueTime, Id, PullRequest, Task};

use crate::flag::Flag;

const TASK: &str = "task-00000000000000c0";
const DUE: &str = "2026-10-01T00:00:00Z";
const ORPHAN: &str = "16";
const RED: &str = "15";

#[derive(Clone, Copy, Debug)]
pub enum Raised {
    Orphan,
    Red,
    Stalled,
}

impl Raised {
    pub fn flag(self) -> Flag {
        let task = || TASK.parse::<Id<Task>>().unwrap();
        match self {
            Self::Orphan => Flag::Orphan(ORPHAN.parse::<PullRequest>().unwrap()),
            Self::Red => Flag::RedUnheld(task(), RED.parse::<PullRequest>().unwrap()),
            Self::Stalled => Flag::Stalled(task(), DUE.parse::<DueTime>().unwrap()),
        }
    }
}
