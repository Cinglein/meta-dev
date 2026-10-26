use std::fmt;

use task::GateStatus;

use crate::task::ci::local_gate::GateState;
use crate::task::ci::step_times::StepTimes;
use crate::task::revision::Commit;

#[derive(Debug)]
pub enum StatusArgument<'post> {
    Endpoint(&'post Commit),
    State(GateState),
    Context(&'post GateStatus),
    Description { version: Commit, times: &'post StepTimes },
}

impl fmt::Display for StatusArgument<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatusArgument::Endpoint(commit) => write!(formatter, "repos/{{owner}}/{{repo}}/statuses/{commit}"),
            StatusArgument::State(state) => write!(formatter, "state={state}"),
            StatusArgument::Context(status) => write!(formatter, "context={status}"),
            StatusArgument::Description { version, times } => {
                write!(formatter, "description=meta-dev {version}: {times}")
            }
        }
    }
}
