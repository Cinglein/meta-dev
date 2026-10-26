mod outcomes;
mod tally;

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};

use outcomes::Outcomes;
use tally::Tally;

use task::CrateName;
use task::Failure;
use task::Site;

#[derive(Debug)]
pub struct MutantCosts {
    tallies: BTreeMap<CrateName, Tally>,
}

impl MutantCosts {
    const OUTCOMES: &str = "outcomes.json";
    const STEP: NonZeroU64 = NonZeroU64::new(100).unwrap();

    pub fn measure(directory: &Path) -> Result<MutantCosts, Failure> {
        let io = |error| Failure::Io { site: Site::File(directory.to_path_buf()), error };
        Self::outcome_files(directory)?
            .iter()
            .map(|file| fs::read_to_string(file).map_err(io))
            .map(|json| Outcomes::parse(&json?))
            .collect::<Result<Vec<Outcomes>, Failure>>()
            .map(Self::tally)
    }

    #[must_use]
    pub fn tally(all: impl IntoIterator<Item = Outcomes>) -> MutantCosts {
        MutantCosts {
            tallies: all.into_iter().fold(BTreeMap::new(), |tallies, outcomes| outcomes.tallied_into(tallies)),
        }
    }

    fn outcome_files(path: &Path) -> Result<Vec<PathBuf>, Failure> {
        let io = |error| Failure::Io { site: Site::File(path.to_path_buf()), error };
        if path.is_dir() {
            fs::read_dir(path)
                .map_err(io)?
                .map(|entry| Self::outcome_files(&entry.map_err(io)?.path()))
                .collect::<Result<Vec<Vec<PathBuf>>, Failure>>()
                .map(|nested| nested.into_iter().flatten().collect())
        } else {
            Ok(path
                .file_name()
                .is_some_and(|name| name == Self::OUTCOMES)
                .then(|| path.to_path_buf())
                .into_iter()
                .collect())
        }
    }
}

impl fmt::Display for MutantCosts {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let overall = self.tallies.values().fold(Tally::default(), |all, tally| all.plus(*tally));
        writeln!(formatter, "[mutant_millis]")?;
        writeln!(formatter, "default = {}", overall.mean().rounded_to(Self::STEP))?;
        writeln!(formatter)?;
        writeln!(formatter, "[mutant_millis.crates]")?;
        self.tallies
            .iter()
            .try_for_each(|(name, tally)| writeln!(formatter, "{name} = {}", tally.mean().rounded_to(Self::STEP)))
    }
}

#[cfg(test)]
mod tests {
    use super::{MutantCosts, Outcomes};
    use std::{env, fs, process};
    const SNIPPET: &str = r#"{"outcomes": [{"scenario": "Baseline", "phase_results": [{"phase": "Build", "duration": 9.0}]}, {"scenario": {"Mutant": {"package": "log", "file": "x"}}, "phase_results": [{"phase": "Build", "duration": 2.56}, {"phase": "Test", "duration": 0.5}]}, {"scenario": {"Mutant": {"package": "state"}}, "phase_results": [{"phase": "Build", "duration": 1.0}]}]}"#;
    const TABLE: &str = "[mutant_millis]\ndefault = 2000\n\n[mutant_millis.crates]\nlog = 3100\nstate = 1000\n";
    const SCRATCH: &str = "meta-costs";
    const NESTED: [&str; 3] = ["a", "b", "outcomes.json"];
    const DECOY: &str = "other.json";

    #[test]
    fn mutant_phases_are_averaged_per_crate_and_overall_then_rounded_to_a_hundred_millis() {
        let costs = MutantCosts::tally([Outcomes::parse(SNIPPET).unwrap()]);
        assert_eq!(costs.to_string(), TABLE);
    }

    #[test]
    fn outcome_files_are_found_under_nested_directories_and_other_files_are_skipped() {
        let root = env::temp_dir().join(format!("{SCRATCH}-{}", process::id()));
        let file = NESTED.iter().fold(root.clone(), |path, part| path.join(part));
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, SCRATCH).unwrap();
        fs::write(root.join(DECOY), SCRATCH).unwrap();
        let found = MutantCosts::outcome_files(&root);
        fs::remove_dir_all(&root).unwrap();
        assert_eq!(found.unwrap(), vec![file]);
    }
}
