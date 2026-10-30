use std::fs::OpenOptions;
use std::io::{self, BufWriter, Write as _};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpawnTrace(PathBuf);

impl SpawnTrace {
    pub const VARIABLE: &'static str = "META_SPAWN_TRACE";
    const STARTS: &'static str = "starts";

    #[must_use]
    pub fn current() -> Option<Self> {
        std::env::var_os(Self::VARIABLE).map(|directory| Self(PathBuf::from(directory)))
    }

    #[must_use]
    pub fn new(directory: &Path) -> Self {
        Self(directory.to_path_buf())
    }

    pub fn noted(&self, command: &Command) -> io::Result<()> {
        let program = Path::new(command.get_program()).file_name().unwrap_or_default().to_string_lossy();
        let mut line = BufWriter::new(OpenOptions::new().create(true).append(true).open(self.0.join(Self::STARTS))?);
        writeln!(line, "{program}")?;
        line.flush()
    }

    pub fn started(command: &Command) -> io::Result<()> {
        Self::current().map_or(Ok(()), |trace| trace.noted(command))
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::SpawnTrace;

    const STARTED: [&str; 2] = ["/bin/sh", "meta"];
    const NOTED: &str = "sh\nmeta\n";

    #[test]
    fn each_start_appends_its_program_name_to_the_trace_in_one_write() {
        let dir = tempfile::tempdir().unwrap();
        let trace = SpawnTrace::new(dir.path());
        STARTED.iter().try_for_each(|program| trace.noted(&Command::new(program))).unwrap();
        let own = std::fs::read_to_string(dir.path().join(SpawnTrace::STARTS));
        let missing = SpawnTrace::new(&dir.path().join(NOTED)).noted(&Command::new(STARTED[1])).is_err();
        assert!(own.is_ok_and(|lines| lines == NOTED) && missing);
    }
}
