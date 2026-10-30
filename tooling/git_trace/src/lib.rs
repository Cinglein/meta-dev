mod invocation;
mod spawns;
mod started;
mod starter;
mod trace_line;

pub use spawns::Spawns;

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::Path;

use started::Started;
use starter::Starter;
use task::{Amount, Failure, Most, Site};
use trace_line::TraceLine;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct GitProcesses(Started<Starter>);

impl GitProcesses {
    pub const TARGET: &'static str = "GIT_TRACE2_EVENT";
    pub const LEAN: [(&'static str, &'static str); 2] =
        [("GIT_TRACE2_EVENT_BRIEF", "true"), ("GIT_TRACE2_EVENT_NESTING", "1")];

    pub fn traced(directory: &Path) -> Result<GitProcesses, Failure> {
        let failed = |error| Failure::Io { site: Site::File(directory.to_path_buf()), error };
        fs::read_dir(directory)
            .map_err(failed)?
            .map(|entry| entry.and_then(|entry| fs::read_to_string(entry.path())).map_err(failed))
            .try_fold(
                Started::default(),
                |counted, trace| Ok(counted.added(TraceLine::starter(&trace?), Most::from(1))),
            )
            .map(GitProcesses)
    }

    #[must_use]
    pub fn amount(&self) -> Amount {
        Amount::Count(self.0.total())
    }
}

impl fmt::Display for GitProcesses {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut most: BTreeMap<&str, Most> = BTreeMap::new();
        let counts = self.0.counts();
        counts.iter().filter_map(|(starter, count)| starter.command().map(|command| (command, *count))).for_each(
            |(command, count)| {
                most.entry(command).and_modify(|seen| *seen = count.max(*seen)).or_insert(count);
            },
        );
        let tests = counts.get(&Starter::Tests).copied().unwrap_or(Most::from(0));
        write!(formatter, "git processes: {} ({tests} not under meta); most per meta run:", self.0.total())?;
        most.iter().try_for_each(|(command, count)| write!(formatter, " {command} {count}"))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::GitProcesses;

    const TRACES: [(&str, &str); 4] = [
        ("a", "{\"event\":\"version\",\"sid\":\"20261029T0-H1-P1\"}\n{\"event\":\"start\"}\n"),
        ("b", "{\"event\":\"version\",\"sid\":\"meta-gc.7.0/20261029T0-H1-P2\"}\n"),
        ("c", "{\"event\":\"version\",\"sid\":\"meta-gc.7.0/20261029T0-H1-P2/20261029T0-H1-P3\"}\n"),
        ("d", "{\"event\":\"vers"),
    ];
    const SHOWN: &str = "git processes: 4 (2 not under meta); most per meta run: gc 2";

    #[test]
    fn every_traced_git_process_counts_once_and_meta_runs_are_told_apart_by_their_parent_sid() {
        let directory = tempfile::tempdir().unwrap();
        TRACES.iter().try_for_each(|(name, trace)| fs::write(directory.path().join(name), trace)).unwrap();
        let traced = GitProcesses::traced(directory.path()).map(|traced| traced.to_string());
        let missing = GitProcesses::traced(&directory.path().join(TRACES[0].0)).is_err();
        assert!(traced.is_ok_and(|shown| shown == SHOWN) && missing);
    }
}
