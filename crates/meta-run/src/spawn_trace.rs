use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Write as _};
use std::path::{Path, PathBuf};

use crate::environment::Environment;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SpawnTrace(Option<PathBuf>);

impl SpawnTrace {
    pub const VARIABLE: &'static str = "META_SPAWN_TRACE";
    pub const TRACED: [&'static str; 3] = [Self::VARIABLE, "GIT_TRACE2_EVENT", "GIT_TRACE2_PARENT_SID"];
    pub const STARTS: &'static str = "starts";

    #[must_use]
    pub fn within(environment: &Environment) -> Self {
        Self(environment.variable(Self::VARIABLE).map(PathBuf::from))
    }

    #[must_use]
    pub fn new(directory: &Path) -> Self {
        Self(Some(directory.to_path_buf()))
    }

    pub fn noted(&self, program: &OsStr) -> io::Result<()> {
        self.0.as_ref().map_or(Ok(()), |directory| {
            let program = Path::new(program).file_name().unwrap_or_default().to_string_lossy();
            let file = OpenOptions::new().create(true).append(true).open(directory.join(Self::STARTS))?;
            let mut line = BufWriter::new(file);
            writeln!(line, "{program}")?;
            line.flush()
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::ffi::OsStr;
    use std::time::{Duration, Instant};

    use super::SpawnTrace;
    use crate::env_policy::EnvPolicy;
    use crate::environment::Environment;
    use crate::run::Run;

    const STARTED: [&str; 2] = ["/bin/sh", "meta"];
    const NOTED: &str = "sh\nmeta\nsh\n";
    const PATIENT: Duration = Duration::from_secs(5);

    #[test]
    fn a_run_notes_its_start_in_the_trace_its_environment_names_only_after_it_spawned() {
        let dir = tempfile::tempdir().unwrap();
        let trace = SpawnTrace::new(dir.path());
        STARTED.iter().try_for_each(|program| trace.noted(OsStr::new(program))).unwrap();
        let traced = Environment::from(BTreeMap::from([(SpawnTrace::VARIABLE.into(), Some(dir.path().into()))]));
        let run =
            |program| Run::inheriting(program, EnvPolicy::Only(&[]), traced.clone()).ran(Instant::now() + PATIENT);
        assert!(run(STARTED[0]).is_ok() && run(NOTED).is_err());
        let own = std::fs::read_to_string(dir.path().join(SpawnTrace::STARTS));
        let missing = SpawnTrace::new(&dir.path().join(NOTED)).noted(OsStr::new(STARTED[1])).is_err();
        let untraced = SpawnTrace::within(&Environment::default()).noted(OsStr::new(NOTED)).is_ok();
        assert!(own.is_ok_and(|lines| lines == NOTED) && missing && untraced);
    }
}
