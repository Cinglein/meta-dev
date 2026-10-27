use std::fmt;

use meta_pull_query::WorkflowName;

#[derive(Clone, Debug)]
pub struct UnseenRun<'name>(&'name WorkflowName);

impl<'name> UnseenRun<'name> {
    #[must_use]
    pub fn new(workflow: &'name WorkflowName) -> Self {
        Self(workflow)
    }
}

impl fmt::Display for UnseenRun<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "no completed {} run on the default branch", self.0)
    }
}
