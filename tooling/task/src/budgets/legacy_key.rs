use strum::{Display, EnumIter};

use super::budget::{Measure, MutationJob};
use super::budget_id::BudgetId;
use crate::budget::Interval;
use crate::policy::{Floor, Limit, Share};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display, EnumIter)]
pub enum LegacyKey {
    #[strum(to_string = "millis.total")]
    Total,
    #[strum(to_string = "millis.per_test")]
    PerTest,
    #[strum(to_string = "millis.full_job")]
    FullJob,
    #[strum(to_string = "millis.diff_job")]
    DiffJob,
    #[strum(to_string = "floors.price_tolerance")]
    PriceTolerance,
    #[strum(to_string = "shares.test_line_percent")]
    TestLinePercent,
    #[strum(to_string = "limits.scc_items")]
    SccItems,
    #[strum(to_string = "baseline")]
    Baseline,
}

impl LegacyKey {
    #[must_use]
    pub const fn measure(self) -> Measure {
        match self {
            LegacyKey::Total => Measure::TestWallTime,
            LegacyKey::PerTest => Measure::SlowestTest,
            LegacyKey::FullJob => Measure::WorkflowWallTime(MutationJob::FullJob),
            LegacyKey::DiffJob => Measure::WorkflowWallTime(MutationJob::DiffJob),
            LegacyKey::PriceTolerance => Measure::MutantPrice,
            LegacyKey::TestLinePercent => Measure::TestLineShare,
            LegacyKey::SccItems => Measure::TangleItems,
            LegacyKey::Baseline => Measure::LintFindings,
        }
    }

    #[must_use]
    pub fn budget_id(self) -> BudgetId {
        let name = match self {
            LegacyKey::Total => "test-time",
            LegacyKey::PerTest => "slowest-test",
            LegacyKey::FullJob => "mutants-full-job",
            LegacyKey::DiffJob => "mutants-diff-job",
            LegacyKey::PriceTolerance => "mutant-price",
            LegacyKey::TestLinePercent => "test-line-share",
            LegacyKey::SccItems => "tangle-items",
            LegacyKey::Baseline => "lint-findings",
        };
        BudgetId::from(name.to_owned())
    }

    #[must_use]
    pub const fn of_interval(interval: Interval) -> Option<LegacyKey> {
        match interval {
            Interval::Total => Some(LegacyKey::Total),
            Interval::PerTest => Some(LegacyKey::PerTest),
            Interval::FullJob => Some(LegacyKey::FullJob),
            Interval::DiffJob => Some(LegacyKey::DiffJob),
            Interval::SampleEvery
            | Interval::DiscoverEvery
            | Interval::PerCrate
            | Interval::TestTimeout
            | Interval::JobSetup => None,
        }
    }

    #[must_use]
    pub const fn of_floor(floor: Floor) -> Option<LegacyKey> {
        match floor {
            Floor::PriceTolerance => Some(LegacyKey::PriceTolerance),
            Floor::FnName
            | Floor::LifetimeName
            | Floor::CutSide
            | Floor::PriceSample
            | Floor::DiffFactor => None,
        }
    }

    #[must_use]
    pub const fn of_share(share: Share) -> Option<LegacyKey> {
        match share {
            Share::TestLinePercent => Some(LegacyKey::TestLinePercent),
            Share::TestsPerHundredFiles | Share::CutConductance | Share::ShardHeadroom => None,
        }
    }

    #[must_use]
    pub const fn of_limit(limit: Limit) -> Option<LegacyKey> {
        match limit {
            Limit::SccItems => Some(LegacyKey::SccItems),
            Limit::Parameters
            | Limit::Nesting
            | Limit::Reexports
            | Limit::PrivateFnsPerType
            | Limit::EdgesPerVertex
            | Limit::TestsPerFile
            | Limit::AssertionsPerTest
            | Limit::LinesPerTest
            | Limit::LiteralsPerTest
            | Limit::LargestInteger
            | Limit::SlowestShown
            | Limit::ReferenceSpawns
            | Limit::TypesPerMethodShape => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use meta_golden::Golden;
    use settings::EmbeddedDefaults;

    use crate::budget::{Interval, Millis};
    use crate::percent::Percent;
    use crate::policy::{Floor, Policy};

    const MILLIS: &str = "[millis]\n";
    const OLD: &str = "[millis]\ntotal = 8000\n";
    const NEW: &str = "[budgets.test-time]\ndivision = \"performance-and-cost\"\nmeasure = \"test_wall_time\"\nallowance = { duration = 8000 }\nwindow = \"per_change\"\nsource = \"ci\"\n";
    const PRICE: &str = "[budgets.mutant-price]\ndivision = \"performance-and-cost\"\nmeasure = \"mutant_price\"\nallowance = { share = 100 }\ntolerance = 25\nwindow = \"per_change\"\nsource = \"ci\"\n";
    const FLOORS: &str = "[floors]\n";
    const OLD_FLOORS: &str = "[floors]\nprice_tolerance = 25\n";
    const NOT_A_TABLE: &str = "millis = 5\n";
    const SPARE: &str = "[spare]\n";
    const DROPPED: &str = "";
    const PRICED: Millis = Millis::new(64);

    impl Policy {
        fn refusal(text: &str) -> Option<String> {
            toml::from_str::<Policy>(text)
                .err()
                .map(|error| error.message().to_owned())
        }

        fn unbudgeted() -> String {
            Policy::DEFAULTS.replace(NEW, DROPPED)
        }

        fn old() -> Policy {
            let unbudgeted = Policy::unbudgeted().replace(PRICE, DROPPED);
            toml::from_str(&unbudgeted.replace(MILLIS, OLD).replace(FLOORS, OLD_FLOORS)).unwrap()
        }

        fn timed_and_priced(&self) -> [Millis; 3] {
            let floor = Percent::from(usize::from(self.floor(Floor::PriceTolerance)));
            [
                self.interval(Interval::Total),
                self.tolerated(PRICED),
                floor.portion_of(PRICED),
            ]
        }
    }

    #[test]
    fn an_old_key_reads_as_its_budget_did_and_is_noted_as_deprecated() {
        let old = Policy::old();
        assert_eq!(
            old.timed_and_priced(),
            Policy::defaults().timed_and_priced()
        );
        let derived = old
            .deprecated()
            .iter()
            .map(|key| (key, old.budgets().budget(&key.budget_id())));
        let shown = format!(
            "{:#?}\n{:?}",
            derived.collect::<Vec<_>>(),
            Policy::defaults().deprecated()
        );
        Golden::for_test(module_path!(), "derived")
            .checked(&shown)
            .unwrap();
    }

    #[test]
    fn an_old_key_and_its_budget_together_or_neither_are_refused() {
        let flat = format!("{NOT_A_TABLE}{}", Policy::DEFAULTS.replace(MILLIS, SPARE));
        let texts = [
            Policy::DEFAULTS.replace(MILLIS, OLD),
            Policy::unbudgeted(),
            flat,
        ];
        let refusals = texts.map(|text| Policy::refusal(&text));
        Golden::for_test(module_path!(), "refused")
            .checked(&format!("{refusals:#?}"))
            .unwrap();
    }
}
