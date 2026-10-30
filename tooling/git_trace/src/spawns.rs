use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use task::{Failure, Most, Site};

use super::GitProcesses;
use super::started::Started;

#[derive(Debug, Default)]
pub struct Spawns {
    tests: BTreeMap<String, Started<String>>,
    git: GitProcesses,
}

impl Spawns {
    pub const GIT_TRACES: &'static str = "git-traces";
    const GIT: &'static str = concat!("gi", "t");

    pub fn traced(root: &Path) -> Result<Spawns, Failure> {
        let tests = Self::entries(root)?;
        tests.iter().filter(|path| path.is_dir()).try_fold(Spawns::default(), |spawns, test| spawns.read(test))
    }

    fn entries(directory: &Path) -> Result<Vec<PathBuf>, Failure> {
        let failed = |error| Failure::Io { site: Site::File(directory.to_path_buf()), error };
        let paths = fs::read_dir(directory).map_err(failed)?.map(|entry| entry.map(|entry| entry.path()));
        paths.collect::<Result<Vec<PathBuf>, _>>().map_err(failed)
    }

    fn read(mut self, test: &Path) -> Result<Spawns, Failure> {
        let git = GitProcesses::traced(&test.join(Self::GIT_TRACES))?;
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
    use std::fs;

    use task::Most;

    use super::Spawns;

    const DIRECTORIES: [&str; 2] = ["it a/git-traces", "it b/git-traces"];
    const FILES: [(&str, &str); 3] = [("it a/7", "git\nmeta\n"), ("it a/8", "git\n"), ("it a/git-traces/t", TRACE)];
    const TRACE: &str = "{\"event\":\"version\",\"sid\":\"20261029T0-H1-P1\"}\n";
    const SHOWN: &str =
        "processes started per test, 1 of 2 tests start any:\n   3 it a: git 2, meta 1\ngit processes: 1";
    const TOTALS: [(&str, usize); 2] = [("it a", 3), ("it b", 0)];

    #[test]
    fn each_test_counts_its_starts_with_git_from_the_larger_of_its_two_traces() {
        let root = tempfile::tempdir().unwrap();
        DIRECTORIES.iter().try_for_each(|directory| fs::create_dir_all(root.path().join(directory))).unwrap();
        FILES.iter().try_for_each(|(path, text)| fs::write(root.path().join(path), text)).unwrap();
        let spawns = Spawns::traced(root.path()).unwrap();
        let totals = BTreeMap::from(TOTALS.map(|(test, count)| (test, Most::from(count))));
        assert!(spawns.to_string().starts_with(SHOWN) && spawns.per_test() == totals, "{spawns}");
        assert!(Spawns::traced(&root.path().join(FILES[0].0)).is_err());
    }
}
