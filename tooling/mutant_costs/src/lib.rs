mod outcomes;
mod overrun;
mod tally;

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};

use outcomes::Outcomes;
use overrun::Overrun;
use tally::Tally;

use task::CrateName;
use task::Failure;
use task::Floor;
use task::Interval;
use task::Millis;
use task::Policy;
use task::Site;

#[derive(Debug, Default)]
pub struct MutantCosts {
    prices: BTreeMap<CrateName, Millis>,
    unsampled: BTreeMap<CrateName, Millis>,
    baseline: Millis,
}

impl MutantCosts {
    const OUTCOMES: &str = "outcomes.json";
    const STEP: NonZeroU64 = NonZeroU64::new(100).unwrap();

    pub fn measure(runs: &[PathBuf], policy: &Policy) -> Result<MutantCosts, Failure> {
        runs.iter()
            .map(|run| Self::read_run(run))
            .collect::<Result<Vec<Vec<Outcomes>>, Failure>>()
            .map(|runs| Self::across(runs, policy))
    }

    #[must_use]
    pub fn across(runs: impl IntoIterator<Item = Vec<Outcomes>>, policy: &Policy) -> MutantCosts {
        runs.into_iter().fold(MutantCosts::default(), |costs, run| costs.costliest_run(&run, policy))
    }

    fn read_run(directory: &Path) -> Result<Vec<Outcomes>, Failure> {
        let io = |error| Failure::Io { site: Site::File(directory.to_path_buf()), error };
        Self::outcome_files(directory)?
            .iter()
            .map(|file| fs::read_to_string(file).map_err(io))
            .map(|json| Outcomes::parse(&json?))
            .collect()
    }

    #[must_use]
    pub fn below_sample_floor(&self) -> String {
        let spelled: Vec<String> =
            self.unsampled.iter().map(|(name, mean)| format!("{name} {}", mean.rounded_up_to(Self::STEP))).collect();
        spelled.join(", ")
    }

    pub fn priced_within(&self, policy: &Policy, job: Interval) -> Result<(), Failure> {
        let under: Vec<String> = self
            .above_price(policy, job)
            .filter(|overrun| overrun.is_beyond_tolerance(policy))
            .map(|overrun| format!("{overrun} beyond tolerance"))
            .collect();
        under.is_empty().then_some(()).ok_or_else(|| Failure::Underpriced(under.join(", ")))
    }

    #[must_use]
    pub fn within_tolerance(&self, policy: &Policy, job: Interval) -> String {
        let spelled: Vec<String> = self
            .above_price(policy, job)
            .filter(|overrun| !overrun.is_beyond_tolerance(policy))
            .map(|overrun| format!("{overrun}, within tolerance"))
            .collect();
        spelled.join(", ")
    }

    fn above_price<'costs>(
        &'costs self,
        policy: &'costs Policy,
        job: Interval,
    ) -> impl Iterator<Item = Overrun> + 'costs {
        let baseline = Overrun::new("baseline".to_owned(), self.baseline, policy.interval(Interval::PerCrate));
        self.prices
            .iter()
            .map(move |(name, price)| Overrun::new(name.to_string(), *price, policy.mutant_price(name, job)))
            .chain([baseline])
            .filter(Overrun::is_above_price)
    }

    fn costliest_run(mut self, run: &[Outcomes], policy: &Policy) -> MutantCosts {
        let floor = policy.floor(Floor::PriceSample);
        self.baseline = run.iter().map(Outcomes::baseline).fold(self.baseline, Millis::max);
        let pooled = run.iter().fold(BTreeMap::new(), |tallies, slice| slice.tallied_into(tallies));
        pooled.into_iter().fold(self, |mut costs, (name, tally): (CrateName, Tally)| {
            let book = if floor.allows(tally.count().into()) { &mut costs.prices } else { &mut costs.unsampled };
            let price = book.entry(name).or_default();
            *price = (*price).max(tally.mean());
            costs
        })
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
    use super::{Failure, Interval, MutantCosts, Outcomes};
    use settings::EmbeddedDefaults;
    use std::{env, fs, iter, process};
    use task::Policy;
    const LISTED: &str = "meta-cli";
    const SAMPLE: usize = 30;
    const AFFECTED: &str = "affected";
    const CASES: [(f64, f64, Interval, bool, bool); 10] = [
        (2.8, 1.0, Interval::DiffJob, true, false),
        (5.9, 1.0, Interval::DiffJob, true, true),
        (6.1, 1.0, Interval::DiffJob, false, false),
        (1.6, 1.0, Interval::FullJob, true, false),
        (1.76, 1.0, Interval::FullJob, true, true),
        (2.0, 1.0, Interval::FullJob, true, true),
        (2.08, 1.0, Interval::FullJob, false, false),
        (2.8, 1.0, Interval::FullJob, false, false),
        (1.6, 16.5, Interval::FullJob, true, true),
        (1.6, 19.5, Interval::FullJob, false, false),
    ];
    const SMALL_DEAR_DIFF: &[(&str, f64, usize)] = &[(AFFECTED, 4.0, BELOW_SAMPLE)];
    const BELOW_SAMPLE: usize = 29;
    const BASELINE: &str = r#"{"scenario": "Baseline", "phase_results": [{"phase": "Build", "duration": 15.0}]}"#;
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
    const SLOW_BASELINE: &str =
        r#"{"outcomes": [{"scenario": "Baseline", "phase_results": [{"phase": "Build", "duration": 60.0}]}]}"#;

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
        let costs = MutantCosts::across(RUNS.map(|crates| vec![run(crates)]), &policy);
        assert_eq!((costs.to_string(), costs.below_sample_floor()), (TABLE.to_owned(), SMALL.to_owned()));
        let sliced = MutantCosts::across([iter::repeat_with(|| run(SLICE)).take(SLICES).collect()], &policy);
        assert!(sliced.to_string().ends_with(SLICED_PRICE));
        let small = MutantCosts::across([vec![run(SMALL_DEAR_DIFF)]], &policy);
        assert!(small.priced_within(&policy, Interval::DiffJob).is_ok());
    }

    #[test]
    fn a_run_may_cost_up_to_price_plus_tolerance_and_a_diff_run_the_price_times_the_diff_factor_and_is_reported_above_price()
     {
        let policy = Policy::defaults();
        let judged = |(seconds, baseline, job, _, _): &(f64, f64, Interval, bool, bool)| {
            let one = format!(
                r#"{{"scenario": {{"Mutant": {{"package": "{AFFECTED}"}}}}, "phase_results": [{{"phase": "Test", "duration": {seconds}}}]}}"#
            );
            let base = format!(
                r#"{{"scenario": "Baseline", "phase_results": [{{"phase": "Build", "duration": {baseline}}}]}}"#
            );
            let run = format!(r#"{{"outcomes": [{base}, {}]}}"#, vec![one; SAMPLE].join(", "));
            let costs = MutantCosts::across([vec![Outcomes::parse(&run).unwrap()]], &policy);
            (costs.priced_within(&policy, *job).is_ok(), !costs.within_tolerance(&policy, *job).is_empty())
        };
        assert!(CASES.iter().all(|case| judged(case) == (case.3, case.4)));
    }

    #[test]
    fn nested_outcome_files_make_one_run_and_a_baseline_dearer_than_the_crate_overhead_fails() {
        let policy = Policy::defaults();
        let root = env::temp_dir().join(format!("{SCRATCH}-{}", process::id()));
        let file = NESTED.iter().fold(root.clone(), |path, part| path.join(part));
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, SLOW_BASELINE).unwrap();
        fs::write(root.join(DECOY), SCRATCH).unwrap();
        let found = MutantCosts::outcome_files(&root);
        let run = MutantCosts::read_run(&root);
        fs::remove_dir_all(&root).unwrap();
        assert_eq!(found.unwrap(), vec![file]);
        let costs = MutantCosts::across([run.unwrap()], &policy);
        assert!(matches!(costs.priced_within(&policy, Interval::FullJob), Err(Failure::Underpriced(_))));
    }
}
