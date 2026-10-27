use std::fmt;
use std::path::Path;

use task::{GateEnvName, GateStatus};

use crate::task::ci::local_gate::{GateState, MetaRev, RecordedTimes, StatusDescription, Toolchain};
use crate::task::revision::Commit;
use crate::task::surfaces::Changed;

#[derive(Debug)]
pub enum StatusArgument<'post> {
    Endpoint(&'post Commit),
    State(GateState),
    Context {
        status: &'post GateStatus,
        scope: &'post Path,
    },
    Description {
        version: MetaRev,
        state: GateState,
        toolchain: &'post Toolchain,
        times: &'post RecordedTimes,
        surfaces: &'post Changed,
        passed: &'post [GateEnvName],
    },
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
            StatusArgument::Description { version, state, toolchain, times, surfaces, passed } => {
                let names: Vec<&str> = passed.iter().map(GateEnvName::as_str).collect();
                let surfaced = if surfaces.is_empty() { "" } else { Self::SURFACES };
                let description = StatusDescription::describing(&version.to_string(), &toolchain.to_string())
                    .after(*state, &times.to_string())
                    .surfacing(surfaced)
                    .passing(&names);
                write!(formatter, "description={description}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use settings::EmbeddedDefaults;
    use task::{GateEnvName, Policy};

    use super::StatusArgument;
    use crate::task::ci::local_gate::{GateState, RecordedTimes, Toolchain};
    use crate::task::surfaces::Changed;

    const VERSION: &str = "0a1b2c";
    const ANY: &str = "Cargo.toml";
    const NAMES: [&str; 2] = ["CHROME", "TERM"];
    const ROOT_CONTEXT: &str = "context=local-gate";
    const UNNOTED: &str = "description=meta-dev 0a1b2c on 1.90-x86_64-unknown-linux-gnu: ";
    const TOOLCHAIN: &str = "1.90-x86_64-unknown-linux-gnu (default)";
    const ROOT: &str = "";
    const NOTED: &str = "; surfaces changed; env passed: CHROME, TERM";

    #[test]
    fn the_root_workspace_keeps_the_bare_status_and_passed_names_follow_the_surfaces_note_only_when_changed() {
        let policy = Policy::defaults();
        let context = StatusArgument::Context { status: policy.gate().unwrap(), scope: Path::new(ROOT) };
        let (times, toolchain): (_, Toolchain) = (RecordedTimes::new(PathBuf::new()), TOOLCHAIN.parse().unwrap());
        let passed: Vec<GateEnvName> = NAMES.iter().map(|name| name.to_string().try_into().unwrap()).collect();
        let shown = |surfaces: &Changed, passed: &[GateEnvName]| {
            let (version, state) = (VERSION.parse().unwrap(), GateState::Success);
            StatusArgument::Description { version, state, toolchain: &toolchain, times: &times, surfaces, passed }
                .to_string()
        };
        assert_eq!(context.to_string(), ROOT_CONTEXT);
        assert_eq!(shown(&Changed::new(Vec::new()), &[]), UNNOTED);
        assert_eq!(shown(&Changed::new(vec![PathBuf::from(ANY)]), &passed), format!("{UNNOTED}{NOTED}"));
    }
}
