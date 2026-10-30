use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use task::{Failure, FailureKind, FailureSet, Most, Printed, Site};

use super::GitProcesses;
use super::started::Started;

#[derive(Debug, Default)]
pub struct Spawns {
    tests: BTreeMap<String, Started<String>>,
    git: GitProcesses,
}

impl Spawns {
    pub const GIT_TRACES: &'static str = "git-traces";
    pub const SEALED_GIT_TRACES: &'static str = "sealed-git-traces";
    const GIT: &'static str = concat!("gi", "t");
    const UNTRACED: &'static str = "traced-tests";

    pub fn traced(root: &Path) -> Result<Spawns, Failure> {
        let outside = Spawns { git: GitProcesses::traced(&root.join(Self::GIT_TRACES))?, ..Spawns::default() };
        let tests = Self::entries(root)?;
        let tests = tests.iter().filter(|path| path.is_dir() && !path.ends_with(Self::GIT_TRACES));
        tests.into_iter().try_fold(outside, |spawns, test| spawns.read(test))
    }

    pub fn covering(self, timed: impl IntoIterator<Item = impl fmt::Display>) -> Result<Spawns, Failure> {
        let timed: BTreeSet<String> = timed.into_iter().map(|test| test.to_string()).collect();
        let traced: BTreeSet<String> = self.tests.keys().cloned().collect();
        let differing: Vec<&String> = traced.symmetric_difference(&timed).collect();
        match differing.first() {
            None => Ok(self),
            Some(_) => Err(Self::untraced(format!("traced and timed tests differ: {differing:?}"))),
        }
    }

    fn untraced(text: String) -> Failure {
        let found = FailureSet::default().recording(FailureKind::Infra { job: Self::UNTRACED.into() });
        Failure::Found { printed: Printed::Text(text), found }
    }

    fn entries(directory: &Path) -> Result<Vec<PathBuf>, Failure> {
        let failed = |error| Failure::Io { site: Site::File(directory.to_path_buf()), error };
        let paths = fs::read_dir(directory).map_err(failed)?.map(|entry| entry.map(|entry| entry.path()));
        paths.collect::<Result<Vec<PathBuf>, _>>().map_err(failed)
    }

    fn read(mut self, test: &Path) -> Result<Spawns, Failure> {
        let late = Self::entries(&test.join(Self::GIT_TRACES))?;
        let ended = |_| Err(Self::untraced(format!("{} started git after it ended", test.display())));
        late.first().map_or(Ok(()), ended)?;
        let git = GitProcesses::traced(&test.join(Self::SEALED_GIT_TRACES))?;
        let failed = |error| Failure::Io { site: Site::File(test.to_path_buf()), error };
        let read = Self::entries(test)?.into_iter().filter(|path| path.is_file()).map(fs::read_to_string);
        let texts = read.collect::<Result<Vec<String>, _>>().map_err(failed)?;
        let started = texts
            .iter()
            .flat_map(|text| text.lines())
            .fold(Started::default(), |started, program| started.added(program.to_owned(), Most::from(1)));
        let spawned = started.counts().get(Self::GIT).copied().unwrap_or(Most::from(0));
        let unseen = Most::from(git.0.total().count().saturating_sub(spawned.count()));
        let name = test.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
        self.tests.insert(name, started.added(Self::GIT.to_owned(), unseen));
        Ok(Spawns { git: GitProcesses(self.git.0.joined(git.0)), ..self })
    }

    #[must_use]
    pub fn git(&self) -> &GitProcesses {
        &self.git
    }

    #[must_use]
    pub fn per_test(&self) -> BTreeMap<&str, Most> {
        self.tests.iter().map(|(test, started)| (test.as_str(), started.total())).collect()
    }
}

impl fmt::Display for Spawns {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut spawning: Vec<(&String, &Started<String>)> =
            self.tests.iter().filter(|(_, started)| started.total().count() > 0).collect();
        spawning.sort_by_key(|(_, started)| Reverse(started.total()));
        writeln!(formatter, "processes started per test, {} of {} tests start any:", spawning.len(), self.tests.len())?;
        spawning
            .iter()
            .try_for_each(|(test, started)| writeln!(formatter, "{:>4} {test}: {started}", started.total()))?;
        write!(formatter, "{}", self.git)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::ffi::OsStr;
    use std::fs;

    use meta_run::SpawnTrace;
    use task::Most;

    use super::Spawns;

    const TESTS: [&str; 3] = ["p it a", "p it b", "p it c"];
    const UNTRACED: &str = "p it d";
    const STARTS: [(usize, &str); 4] = [(0, Spawns::GIT), (0, "meta"), (0, Spawns::GIT), (2, "meta")];
    const TRACE: &str = "{\"event\":\"version\",\"sid\":\"20261029T0-H1-P1\"}\n";
    const SHOWN: &str = "processes started per test, 2 of 3 tests start any:\n   3 p it a: git 2, meta 1\n   1 p it c: meta 1\ngit processes: 2";
    const TOTALS: [(&str, usize); 3] = [("p it a", 3), ("p it b", 0), ("p it c", 1)];

    #[test]
    fn each_test_counts_its_starts_with_git_from_the_larger_of_its_two_traces() {
        let root = tempfile::tempdir().unwrap();
        let outside = root.path().join(Spawns::GIT_TRACES);
        let git_dirs = TESTS.map(|test| root.path().join(test).join(Spawns::SEALED_GIT_TRACES));
        let late_dirs = TESTS.map(|test| root.path().join(test).join(Spawns::GIT_TRACES));
        let [first, quiet, _] = &git_dirs;
        git_dirs.iter().chain(&late_dirs).chain([&outside]).try_for_each(fs::create_dir_all).unwrap();
        [first, &outside].iter().try_for_each(|dir| fs::write(dir.join(Spawns::GIT), TRACE)).unwrap();
        let noted = |(test, program): &(usize, &str)| {
            SpawnTrace::new(&root.path().join(TESTS[*test])).noted(OsStr::new(program))
        };
        STARTS.iter().try_for_each(noted).unwrap();
        let spawns = Spawns::traced(root.path()).unwrap();
        let totals = BTreeMap::from(TOTALS.map(|(test, count)| (test, Most::from(count))));
        assert!(spawns.to_string().starts_with(SHOWN) && spawns.per_test() == totals);
        let extra = Spawns::traced(root.path()).unwrap().covering(TESTS.iter().chain([&UNTRACED])).is_err();
        assert!(Spawns::traced(quiet).is_err() && extra && spawns.covering(TESTS).is_ok());
    }
}
