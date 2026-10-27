use std::convert::Infallible;
use std::str::FromStr;

use meta_schema::{Id, Task};
use task::BranchName;

use crate::task::ci::untracked::Untracked;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeadBranch {
    name: BranchName,
    task: Option<Id<Task>>,
}

impl HeadBranch {
    pub fn task(&self) -> Result<Id<Task>, Untracked> {
        self.task.ok_or_else(|| Untracked::NotTask(self.name.clone()))
    }
}

impl FromStr for HeadBranch {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<HeadBranch, Infallible> {
        let name = text.trim();
        let cut = name.match_indices('-').nth(1).map_or(name.len(), |(at, _)| at);
        let Ok(branch) = name.parse();
        Ok(HeadBranch { name: branch, task: name.get(..cut).and_then(|id| id.parse().ok()) })
    }
}
