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
use task::Millis;
use task::Policy;
use task::Site;

#[derive(Debug)]
pub struct MutantCosts {
    prices: BTreeMap<CrateName, Millis>,
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
        MutantCosts { prices: all.into_iter().fold(BTreeMap::new(), |prices, run| Self::costliest_run(prices, &run)) }
    }

    pub fn priced_within(&self, policy: &Policy) -> Result<(), Failure> {
        let table = policy.mutant_millis();
        let under: Vec<String> = self
            .prices
            .iter()
            .filter(|(name, price)| price.rounded_up_to(Self::STEP) > table.for_crate(name))
            .map(|(name, price)| format!("{name} {price} > {}", table.for_crate(name)))
            .collect();
        under.is_empty().then_some(()).ok_or_else(|| Failure::Underpriced(under.join(", ")))
    }

    fn costliest_run(mut prices: BTreeMap<CrateName, Millis>, run: &Outcomes) -> BTreeMap<CrateName, Millis> {
        run.tallied_into(BTreeMap::new()).into_iter().for_each(|(name, tally): (CrateName, Tally)| {
            let price = prices.entry(name).or_default();
            *price = (*price).max(tally.mean());
        });
        prices
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
        let costliest = self.prices.values().max().copied().unwrap_or_default();
        writeln!(formatter, "[mutant_millis]")?;
        writeln!(formatter, "default = {}", costliest.rounded_up_to(Self::STEP))?;
        writeln!(formatter)?;
        writeln!(formatter, "[mutant_millis.crates]")?;
        self.prices
            .iter()
            .try_for_each(|(name, price)| writeln!(formatter, "{name} = {}", price.rounded_up_to(Self::STEP)))
    }
}

#[cfg(test)]
mod tests {
    use super::{Failure, Millis, MutantCosts, Outcomes};
    use settings::EmbeddedDefaults;
    use std::{env, fs, process};
    use task::Policy;
    const LISTED: &str = "meta-cli";
    const SNIPPET: &str = r#"{"outcomes": [{"scenario": "Baseline", "phase_results": [{"phase": "Build", "duration": 9.0}]}, {"scenario": {"Mutant": {"package": "log", "file": "x"}}, "phase_results": [{"phase": "Build", "duration": 2.51}, {"phase": "Test", "duration": 0.5}]}, {"scenario": {"Mutant": {"package": "state"}}, "phase_results": [{"phase": "Build", "duration": 1.0}]}]}"#;
    const SLOWER_STATE: &str = r#"{"outcomes": [{"scenario": {"Mutant": {"package": "state"}}, "phase_results": [{"phase": "Test", "duration": 2.0}]}]}"#;
    const TABLE: &str = "[mutant_millis]\ndefault = 3100\n\n[mutant_millis.crates]\nlog = 3100\nstate = 2000\n";
    const SCRATCH: &str = "meta-costs";
    const NESTED: [&str; 3] = ["a", "b", "outcomes.json"];
    const DECOY: &str = "other.json";

    #[test]
    fn each_crate_is_priced_at_its_costliest_run_and_unknown_crates_at_the_costliest_crate() {
        let runs = [SNIPPET, SLOWER_STATE, SNIPPET].map(|run| Outcomes::parse(run).unwrap());
        assert_eq!(MutantCosts::tally(runs).to_string(), TABLE);
    }

    #[test]
    fn a_crate_measured_at_its_price_is_within_it_and_one_measured_dearer_is_underpriced() {
        let policy = Policy::defaults();
        let price = policy.mutant_millis().for_crate(&LISTED.parse().unwrap());
        let measured = |millis: Millis| {
            let seconds = millis.duration().as_secs_f64();
            let run = format!(
                r#"{{"outcomes": [{{"scenario": {{"Mutant": {{"package": "{LISTED}"}}}}, "phase_results": [{{"phase": "Test", "duration": {seconds}}}]}}]}}"#
            );
            MutantCosts::tally([Outcomes::parse(&run).unwrap()]).priced_within(&policy)
        };
        assert!(measured(price).is_ok());
        assert!(matches!(measured(price.plus(price)), Err(Failure::Underpriced(_))));
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
