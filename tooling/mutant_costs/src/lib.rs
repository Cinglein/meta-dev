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
use task::Floor;
use task::Millis;
use task::Policy;
use task::Site;

#[derive(Debug, Default)]
pub struct MutantCosts {
    prices: BTreeMap<CrateName, Millis>,
    unsampled: BTreeMap<CrateName, Millis>,
}

impl MutantCosts {
    const OUTCOMES: &str = "outcomes.json";
    const STEP: NonZeroU64 = NonZeroU64::new(100).unwrap();

    pub fn measure(directory: &Path, policy: &Policy) -> Result<MutantCosts, Failure> {
        let io = |error| Failure::Io { site: Site::File(directory.to_path_buf()), error };
        Self::outcome_files(directory)?
            .iter()
            .map(|file| fs::read_to_string(file).map_err(io))
            .map(|json| Outcomes::parse(&json?))
            .collect::<Result<Vec<Outcomes>, Failure>>()
            .map(|all| Self::tally(all, policy))
    }

    #[must_use]
    pub fn tally(all: impl IntoIterator<Item = Outcomes>, policy: &Policy) -> MutantCosts {
        all.into_iter().fold(MutantCosts::default(), |costs, run| costs.costliest_run(&run, policy))
    }

    #[must_use]
    pub fn below_sample_floor(&self) -> String {
        let spelled: Vec<String> =
            self.unsampled.iter().map(|(name, mean)| format!("{name} {}", mean.rounded_up_to(Self::STEP))).collect();
        spelled.join(", ")
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

    fn costliest_run(mut self, run: &Outcomes, policy: &Policy) -> MutantCosts {
        let floor = policy.floor(Floor::PriceSample);
        run.tallied_into(BTreeMap::new()).into_iter().for_each(|(name, tally): (CrateName, Tally)| {
            let book = if floor.allows(tally.count().into()) { &mut self.prices } else { &mut self.unsampled };
            let price = book.entry(name).or_default();
            *price = (*price).max(tally.mean());
        });
        self
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
    use std::{env, fs, iter, process};
    use task::Policy;
    const LISTED: &str = "meta-cli";
    const SAMPLE: usize = 30;
    const BELOW_SAMPLE: usize = 29;
    const BASELINE: &str = r#"{"scenario": "Baseline", "phase_results": [{"phase": "Build", "duration": 9.0}]}"#;
    const RUNS: [&[(&str, f64, usize)]; 3] =
        [&[("log", 3.01, SAMPLE), ("state", 1.0, SAMPLE)], &[("state", 2.0, SAMPLE)], &[("log", 9.0, BELOW_SAMPLE)]];
    const SMALL: &str = "log 9000";
    const SLICE: &[(&str, f64, usize)] = &[(LISTED, 5.8, 25)];
    const SLICES: usize = 11;
    const SLICED_PRICE: &str = "meta-cli = 5800\n";
    const TABLE: &str = "[mutant_millis]\ndefault = 3100\n\n[mutant_millis.crates]\nlog = 3100\nstate = 2000\n";
    const SCRATCH: &str = "meta-costs";
    const NESTED: [&str; 3] = ["a", "b", "outcomes.json"];
    const DECOY: &str = "other.json";

    #[test]
    fn each_sampled_crate_is_priced_at_its_costliest_run_rounded_up_and_small_runs_are_only_reported() {
        let policy = Policy::defaults();
        let run = |crates: &[(&str, f64, usize)]| {
            let mutants = crates.iter().flat_map(|(package, seconds, mutants)| {
                vec![format!(r#"{{"scenario": {{"Mutant": {{"package": "{package}"}}}}, "phase_results": [{{"phase": "Test", "duration": {seconds}}}]}}"#); *mutants]
            });
            let outcomes: Vec<String> = [BASELINE.to_owned()].into_iter().chain(mutants).collect();
            Outcomes::parse(&format!(r#"{{"outcomes": [{}]}}"#, outcomes.join(", "))).unwrap()
        };
        let costs = MutantCosts::tally(RUNS.map(run), &policy);
        assert_eq!((costs.to_string(), costs.below_sample_floor()), (TABLE.to_owned(), SMALL.to_owned()));
        let sliced = MutantCosts::tally(iter::repeat_with(|| run(SLICE)).take(SLICES), &policy);
        assert!(sliced.to_string().ends_with(SLICED_PRICE));
    }

    #[test]
    fn a_sampled_crate_at_its_price_is_within_it_dearer_is_underpriced_and_a_small_run_never_is() {
        let policy = Policy::defaults();
        let price = policy.mutant_millis().for_crate(&LISTED.parse().unwrap());
        let measured = |millis: Millis, mutants: usize| {
            let seconds = millis.duration().as_secs_f64();
            let one = format!(
                r#"{{"scenario": {{"Mutant": {{"package": "{LISTED}"}}}}, "phase_results": [{{"phase": "Test", "duration": {seconds}}}]}}"#
            );
            let run = format!(r#"{{"outcomes": [{}]}}"#, vec![one; mutants].join(", "));
            MutantCosts::tally([Outcomes::parse(&run).unwrap()], &policy).priced_within(&policy)
        };
        assert!(measured(price, SAMPLE).is_ok());
        assert!(matches!(measured(price.plus(price), SAMPLE), Err(Failure::Underpriced(_))));
        assert!(measured(price.plus(price), BELOW_SAMPLE).is_ok());
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
