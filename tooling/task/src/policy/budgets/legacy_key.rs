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
            | Limit::SlowestShown => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use settings::EmbeddedDefaults;

    use super::super::budget::{Budget, Measure};
    use super::LegacyKey;
    use crate::budget::Interval;
    use crate::policy::Policy;

    const MILLIS: &str = "[millis]\n";
    const OLD: &str = "[millis]\ntotal = 4000\n";
    const NEW: &str = "[budgets.test-time]\ndivision = \"performance-and-cost\"\nmeasure = \"test_wall_time\"\nallowance = { duration = 4000 }\nwindow = \"per_change\"\nsource = \"local_gate\"\n";
    const BOTH: &str = "millis.total: also set as a budget; drop the old key";
    const NEITHER: &str = "millis.total: not set; declare it as a budget";

    impl Policy {
        fn refusal(text: &str) -> Option<String> {
            toml::from_str::<Policy>(text)
                .err()
                .map(|error| error.message().to_owned())
        }
    }

    #[test]
    fn an_old_key_reads_as_its_budget_did_and_is_noted_as_deprecated() {
        let old = toml::from_str::<Policy>(&Policy::DEFAULTS.replace(NEW, "").replace(MILLIS, OLD))
            .unwrap();
        assert_eq!(
            old.interval(Interval::Total),
            Policy::defaults().interval(Interval::Total)
        );
        assert!(
            old.deprecated() == [LegacyKey::Total] && Policy::defaults().deprecated().is_empty()
        );
        let derived = old
            .budgets()
            .budget(&LegacyKey::Total.budget_id())
            .map(Budget::measure);
        assert_eq!(derived, Some(Measure::TestWallTime));
    }

    #[test]
    fn an_old_key_and_its_budget_together_or_neither_are_refused() {
        assert_eq!(
            Policy::refusal(&Policy::DEFAULTS.replace(MILLIS, OLD)).as_deref(),
            Some(BOTH)
        );
        assert_eq!(
            Policy::refusal(&Policy::DEFAULTS.replace(NEW, "")).as_deref(),
            Some(NEITHER)
        );
    }
}
