use meta_schema::{Id, PullRequest, Task};
use meta_state::TaskState;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChangeLink {
    pull_request: Option<PullRequest>,
    branch: Option<Id<Task>>,
}

impl ChangeLink {
    #[must_use]
    pub fn new(pull_request: Option<PullRequest>, branch: Option<Id<Task>>) -> ChangeLink {
        ChangeLink {
            pull_request,
            branch,
        }
    }

    #[must_use]
    pub fn links(&self, task: &TaskState) -> bool {
        self.branch == Some(task.new_task().task())
            || self
                .pull_request
                .is_some_and(|pull_request| task.pull_requests().contains(&pull_request))
    }
}
