use meta_schema::{Id, Task};
use serde::Deserialize;

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct HeadBranch(String);

impl HeadBranch {
    #[must_use]
    pub fn task(&self) -> Option<Id<Task>> {
        let stacked = self
            .0
            .rsplit_once('-')
            .filter(|(_, step)| step.bytes().all(|byte| byte.is_ascii_digit()) && !step.is_empty());
        self.0.parse().ok().or_else(|| stacked.and_then(|(task, _)| task.parse().ok()))
    }
}
