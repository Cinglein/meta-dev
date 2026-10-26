use std::fmt;
use std::path::Path;

use task::GateStatus;

use crate::task::ci::local_gate::GateState;
use crate::task::ci::step_times::StepTimes;
use crate::task::revision::Commit;
use crate::task::surfaces::Changed;

#[derive(Debug)]
pub enum StatusArgument<'post> {
    Endpoint(&'post Commit),
    State(GateState),
    Context { status: &'post GateStatus, scope: &'post Path },
    Description { version: Commit, times: &'post StepTimes, surfaces: &'post Changed },
}

impl StatusArgument<'_> {
    const SURFACES: &str = "; surfaces changed";
}

impl fmt::Display for StatusArgument<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatusArgument::Endpoint(commit) => write!(formatter, "repos/{{owner}}/{{repo}}/statuses/{commit}"),
            StatusArgument::State(state) => write!(formatter, "state={state}"),
            StatusArgument::Context { status, scope } if scope.as_os_str().is_empty() => {
                write!(formatter, "context={status}")
            }
            StatusArgument::Context { status, scope } => write!(formatter, "context={status}/{}", scope.display()),
            StatusArgument::Description { version, times, surfaces } => {
                write!(formatter, "description=meta-dev {version}: {times}")?;
                formatter.write_str(if surfaces.is_empty() { "" } else { Self::SURFACES })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use settings::EmbeddedDefaults;
    use task::Policy;

    use super::StatusArgument;
    use crate::task::ci::step_times::StepTimes;
    use crate::task::revision::Commit;
    use crate::task::surfaces::Changed;

    const VERSION: &str = "0a1b2c";
    const ROOT_CONTEXT: &str = "context=local-gate";
    const UNNOTED: &str = "description=meta-dev 0a1b2c: ";

    #[test]
    fn the_root_workspace_keeps_the_bare_status_and_unchanged_surfaces_add_no_note() {
        let policy = Policy::defaults();
        let context = StatusArgument::Context { status: policy.gate().unwrap(), scope: Path::new("") };
        let (times, unchanged) = (StepTimes::default(), Changed::new(Vec::new()));
        let version: Commit = VERSION.parse().unwrap();
        let description = StatusArgument::Description { version, times: &times, surfaces: &unchanged };
        assert_eq!(context.to_string(), ROOT_CONTEXT);
        assert_eq!(description.to_string(), UNNOTED);
    }
}
