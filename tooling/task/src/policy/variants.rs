use settings::EmbeddedDefaults;
use strum::IntoEnumIterator;

use super::{Limit, Policy, Share};
use crate::budget::{Interval, Millis};
use crate::budgets::{Amount, Ledger, LegacyKey};
use crate::percent::Percent;

const OLD: &str = include_str!("old_policy.toml");
const END: &str = "";
const ALL_TESTS: &str = "tests = [\"*\"]";
const SOME_TESTS: &str = "tests = [\"meta_*\"]";
const TIMED: &str = "allowance = { duration = 4000 }\n";
const TANGLES: &str = "allowance = { count = 83 }\n";
const PRICE: &str = "allowance = { share = 100 }\n";
const WINDOW: &str = "window = \"per_change\"\n";
const CI: &str = "source = \"ci\"\n";

const REFUSED: [(&str, &str, &str, &str); 34] = [
    (
        "meta", "\nparameters = 4\n", "\n", "[limits]: parameters is not set",
    ),
    (
        "meta", "\nfn_name = 3\n", "\n", "[floors]: fn_name is not set",
    ),
    (
        "meta", "\ntests_per_hundred_files = 40\n", "\n",
        "[shares]: tests_per_hundred_files is not set",
    ),
    (
        "meta", "\ngranted = 5\n", "\n", "[memory_gib]: granted is not set",
    ),
    (
        "meta", "\nsample_every = 20\n", "\n", "[millis]: sample_every is not set",
    ),
    (
        "meta",
        END,
        "\n[budgets.open-issues]\ndivision = \"features-and-maintenance\"\nmeasure = { open_items = { kind = \"issue\" } }\nallowance = { count = 20 }\nwindow = \"standing\"\n\n[domains.\"features-and-maintenance/x\"]\ncrates = [\"*\"]\n\n[budgets.x-tangle]\ndivision = \"features-and-maintenance/x\"\nmeasure = \"tangle_items\"\nallowance = { of = \"open-issues\", count = 20 }\nwindow = \"per_change\"\n",
        "budget x-tangle: its measure is not the measure of the budget it is part of",
    ),
    (
        "meta",
        END,
        "\n[budgets.a]\ndivision = \"features-and-maintenance/a\"\nmeasure = { open_items = { kind = \"task\" } }\nallowance = { of = \"tangle-items\", percent = 10 }\nwindow = \"standing\"\n",
        "budget a: its measure is not the measure of the budget it is part of",
    ),
    (
        "meta",
        END,
        "\n[budgets.issues]\ndivision = \"features-and-maintenance\"\nmeasure = { open_items = { kind = \"task\" } }\nallowance = { count = 2 }\nwindow = \"standing\"\n\n[budgets.a]\ndivision = \"features-and-maintenance/a\"\nmeasure = { open_items = { kind = \"concern\" } }\nallowance = { of = \"issues\", count = 1 }\nwindow = \"standing\"\n",
        "budget a: its measure is not the measure of the budget it is part of",
    ),
    (
        "meta", WINDOW, "window = \"standing\"\n", "effects take a standing window",
    ),
    (
        "meta",
        END,
        "\n[budgets.issues]\ndivision = \"features-and-maintenance\"\nmeasure = { open_items = { kind = \"task\" } }\nallowance = { count = 2 }\nwindow = \"per_change\"\n",
        "effects take a standing window",
    ),
    (
        "meta",
        END,
        "\n[budgets.issues]\ndivision = \"features-and-maintenance\"\nmeasure = { open_items = { kind = \"task\" } }\nallowance = { count = 2 }\nwindow = \"standing\"\n\n[budgets.more-issues]\ndivision = \"features-and-maintenance\"\nmeasure = { open_items = { kind = \"task\" } }\nallowance = { count = 3 }\nwindow = \"standing\"\n",
        "another top-level budget has its measure",
    ),
    (
        "meta",
        END,
        "\n[budgets.issues]\ndivision = \"features-and-maintenance\"\nmeasure = { open_items = { kind = \"task\" } }\nallowance = { count = 2 }\nwindow = \"standing\"\n\n[budgets.a]\ndivision = \"features-and-maintenance/a\"\nmeasure = { open_items = { kind = \"task\" } }\nallowance = { of = \"issues\", count = 2 }\nwindow = \"standing\"\n\n[budgets.b]\ndivision = \"features-and-maintenance/b\"\nmeasure = { open_items = { kind = \"task\" } }\nallowance = { of = \"issues\", count = 1 }\nwindow = \"standing\"\n",
        "budget issues: its parts add up to more than all of it",
    ),
    (
        "meta",
        END,
        "\n[budgets.issues]\ndivision = \"features-and-maintenance\"\nmeasure = { open_items = { kind = \"task\" } }\nallowance = { count = 2 }\nwindow = \"standing\"\n\n[budgets.a]\ndivision = \"features-and-maintenance/a\"\nmeasure = { open_items = { kind = \"task\" } }\nallowance = { of = \"issues\", count = 1 }\nwindow = \"standing\"\n\n[budgets.b]\ndivision = \"features-and-maintenance/b\"\nmeasure = { open_items = { kind = \"task\" } }\nallowance = { of = \"issues\", percent = 10 }\nwindow = \"standing\"\n",
        "budget issues: its parts are split by both percent and count",
    ),
    (
        "meta",
        END,
        "\n[domains.\"performance-and-cost/x\"]\ntests = [\"meta_*\"]\n\n[budgets.x-time]\ndivision = \"performance-and-cost/x\"\nmeasure = \"test_wall_time\"\nallowance = { of = \"test-time\", count = 50 }\nwindow = \"per_change\"\n",
        "budget x-time: its allowance is not in its measure's unit",
    ),
    (
        "meta", WINDOW, "window = { rolling = 604800000 }\n", "unknown variant `rolling`",
    ),
    (
        "meta", CI, "source = \"local_gate\"\n", "unknown variant `local_gate`",
    ),
    (
        "meta",
        END,
        "\n[budgets.x-findings]\ndivision = \"performance-and-cost/x\"\nmeasure = \"lint_findings\"\nallowance = { of = \"test-time\", percent = 10 }\nwindow = \"per_change\"\nsource = \"ci\"\n",
        "its allowance is not in its measure's unit",
    ),
    (
        "meta",
        END,
        "\n[budgets.a-time]\ndivision = \"performance-and-cost\"\nmeasure = \"test_wall_time\"\nallowance = { duration = 9000 }\nwindow = \"per_change\"\nsource = \"ci\"\n",
        "another top-level budget has its measure",
    ),
    (
        "old",
        END,
        "\n[domains.performance-and-cost]\ntests = [\"greeting_*\"]\nworkflows = [\"*\"]\n",
        "covers only part of tests",
    ),
    ("meta", ALL_TESTS, SOME_TESTS, "covers only part of tests"),
    (
        "meta", CI, "source = \"ci\"\noverspend = \"escalate\"\n", "unknown variant `escalate`",
    ),
    (
        "meta", TANGLES, "allowance = { count = 83 }\ntolerance = 50\n",
        "a lint enforces its measure; it takes no tolerance",
    ),
    (
        "meta",
        TIMED,
        "allowance = { duration = 4000 }\ntolerance = 9223372036854775807\n",
        "its blocking line overflows",
    ),
    (
        "meta",
        TIMED,
        "allowance = { duration = 4000 }\ntolerance = 18446744073709551615\n",
        "",
    ),
    (
        "meta", TIMED, "allowance = { duration = 4000 }\nwarning = 100\n",
        "must be below the blocking line",
    ),
    (
        "old", END, "\n[domains.performance-and-cost]\nworkflows = [\"*\"]\n", "needs tests",
    ),
    (
        "meta",
        END,
        "\n[domains.\"performance-and-cost/x\"]\ntests = [\"*\", \"a*\"]\nworkflows = [\"*\",\"b\"]\n",
        "not strictly inside",
    ),
    (
        "meta",
        END,
        "\n[domains.\"performance-and-cost/x\"]\ntests = [\"a*\"]\n\n[domains.\"performance-and-cost/x/y\"]\ntests = [\"a*\"]\n",
        "not strictly inside",
    ),
    (
        "meta",
        END,
        "\n[domains.\"performance-and-cost/x\"]\ntests = [\"a*\"]\n\n[domains.\"performance-and-cost/x/y\"]\ntests = [\"b*\"]\n",
        "not strictly inside",
    ),
    (
        "meta",
        END,
        "\n[domains.\"performance-and-cost/x\"]\ntests = [\"a\"]\n\n[domains.\"performance-and-cost/x/y\"]\ntests = [\"a*\"]\n",
        "not strictly inside",
    ),
    (
        "meta",
        END,
        "\n[budgets.x-findings]\ndivision = \"features-and-maintenance/x\"\nmeasure = \"lint_findings\"\nallowance = { of = \"lint-findings\", percent = 10 }\nwindow = \"per_change\"\nsource = \"ci\"\n\n[domains.\"features-and-maintenance/x\"]\npaths = [\"crates*\"]\n",
        "a baseline budget has no parts",
    ),
    (
        "old",
        END,
        "\n[budgets.test-time]\ndivision = \"performance-and-cost\"\nmeasure = \"slowest_test\"\nallowance = { duration = 100 }\nwindow = \"per_change\"\nsource = \"ci\"\n",
        "",
    ),
    (
        "meta",
        END,
        "\n[domains.architecture-and-planning]\n\n[domains.\"architecture-and-planning/x\"]\ntests = [\"a\"]\n",
        "planning has no code scopes",
    ),
    (
        "meta",
        END,
        "\n[budgets.typo]\ndivision = \"features-and-maintenance\"\nmeasure = { open_items = { severity = { at_level = \"critcal\" } } }\nallowance = { count = 0 }\nwindow = \"standing\"\n",
        "critcal is not a level of its division's scale",
    ),
];

const ACCEPTED: [(&str, &str, &str); 6] = [
    (
        "old",
        END,
        "\n[domains.\"performance-and-cost/x\"]\ntests = [\"*\"]\n\n[budgets.x-time]\ndivision = \"performance-and-cost/x\"\nmeasure = \"test_wall_time\"\nallowance = { of = \"test-time\", percent = 50 }\nwindow = \"per_change\"\n",
    ),
    ("meta", END, END),
    ("old", END, END),
    ("meta", PRICE, "allowance = { share = 50 }\n"),
    ("meta", CI, "source = \"ci\"\nwarning = 0\n"),
    (
        "meta", TIMED, "allowance = { duration = 4000 }\ntolerance = 10\n",
    ),
];

impl Policy {
    fn variant(base: &str, found: &str, put: &str) -> Result<Policy, toml::de::Error> {
        let text = if base == "old" { OLD } else { Policy::DEFAULTS };
        let text = if found.is_empty() {
            format!("{text}{put}")
        } else {
            text.replacen(found, put, 1)
        };
        toml::from_str::<Policy>(&text)
    }

    fn disagreements(&self) -> Vec<String> {
        LegacyKey::iter()
            .filter(|key| self.enforced(*key).map(|amount| amount.to_string()) != self.tabled(*key))
            .map(|key| key.to_string())
            .collect()
    }
}

#[test]
fn every_variant_that_drops_or_loosens_a_limit_is_refused_when_read() {
    let refused = REFUSED.map(|(base, found, put, _)| {
        Policy::variant(base, found, put)
            .err()
            .map(|error| error.message().to_owned())
    });
    let expected = REFUSED.map(|(_, _, _, refusal)| refusal);
    let unrefused: Vec<&str> = expected
        .iter()
        .zip(&refused)
        .filter(|(refusal, got)| !got.as_ref().is_some_and(|got| got.contains(*refusal)))
        .map(|(refusal, _)| *refusal)
        .collect();
    assert!(unrefused.is_empty(), "{unrefused:?}\n{refused:?}");
}

const PRICED: Millis = Millis::new(64);
const PERCENT: char = '%';

impl Policy {
    fn enforced(&self, key: LegacyKey) -> Option<Amount> {
        let millis = |interval: Interval| Some(Amount::Duration(self.interval(interval)));
        match key {
            LegacyKey::Total => millis(Interval::Total),
            LegacyKey::PerTest => millis(Interval::PerTest),
            LegacyKey::FullJob => millis(Interval::FullJob),
            LegacyKey::DiffJob => millis(Interval::DiffJob),
            LegacyKey::PriceTolerance => Some(Amount::Duration(self.tolerated(PRICED))),
            LegacyKey::TestLinePercent => Some(Amount::Share(self.share(Share::TestLinePercent))),
            LegacyKey::SccItems => Some(Amount::Count(self.limit(Limit::SccItems))),
            LegacyKey::Baseline => None,
        }
    }

    fn tabled(&self, key: LegacyKey) -> Option<String> {
        let shown = Ledger::default().statement(self).to_string();
        let row = shown
            .lines()
            .find(|row| row.split_whitespace().next() == Some(&key.budget_id().to_string()))?;
        let blocking = row.split_whitespace().rev().nth(1)?.to_owned();
        match key {
            LegacyKey::PriceTolerance => blocking
                .trim_end_matches(PERCENT)
                .parse::<usize>()
                .ok()
                .map(|percent| {
                    Amount::Duration(Percent::from(percent).portion_of(PRICED)).to_string()
                }),
            LegacyKey::Baseline => None,
            _ => Some(blocking),
        }
    }
}

#[test]
fn the_kept_forms_are_read_and_the_table_shows_the_line_each_check_enforces() {
    let variants = ACCEPTED.map(|(base, found, put)| Policy::variant(base, found, put));
    let read = variants
        .each_ref()
        .map(|variant| variant.as_ref().err().map(ToString::to_string));
    assert_eq!(read, [None, None, None, None, None, None]);
    let disagreements: Vec<String> = variants
        .iter()
        .flatten()
        .flat_map(Policy::disagreements)
        .collect();
    assert!(disagreements.is_empty(), "{disagreements:?}");
}
