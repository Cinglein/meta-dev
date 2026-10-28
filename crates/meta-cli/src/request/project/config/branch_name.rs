use std::ffi::OsStr;
use std::fmt;
use std::str::FromStr;

use meta_schema::{Id, Task};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BranchName(String);

impl BranchName {
    const HEADS: &'static str = "refs/heads/";

    pub fn task(&self) -> Option<Id<Task>> {
        self.0.split('/').find_map(|segment| segment.parse().ok())
    }

    pub fn qualified(&self) -> String {
        [Self::HEADS, &self.0].concat()
    }
}

impl FromStr for BranchName {
    type Err = std::convert::Infallible;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Ok(Self(name.strip_prefix(Self::HEADS).unwrap_or(name).to_owned()))
    }
}

impl AsRef<OsStr> for BranchName {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(&self.0)
    }
}

impl fmt::Display for BranchName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::BranchName;

    const TASK: &str = "task-53c6698cb2d5ea01";
    const NAMED: [&str; 3] = ["task-53c6698cb2d5ea01", "status/task-53c6698cb2d5ea01", "task-53c6698cb2d5ea01/part"];
    const UNNAMED: [&str; 3] = ["meta-status", "status-task-53c6698cb2d5ea01", "task-53c6698cb2d5ea0"];
    const EMPTY: [&str; 2] = ["", "refs/heads/"];

    #[test]
    fn a_branch_names_a_task_by_a_whole_path_segment_and_an_empty_name_is_no_branch() {
        let task = |name: &str| name.parse::<BranchName>().unwrap().task().map(|task| task.to_string());
        assert!(NAMED.iter().all(|name| task(name).as_deref() == Some(TASK)));
        assert!(UNNAMED.iter().all(|name| task(name).is_none()));
        assert!(EMPTY.iter().all(|name| name.parse::<BranchName>().is_err()));
    }
}
