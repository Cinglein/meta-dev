use std::collections::BTreeSet;

use settings::EmbeddedDefaults;

use super::super::tests::{ADDRESSING, STANDING};
use crate::budgets::{Amount, Ledger, Measure};
use crate::policy::{Most, Policy};
use crate::selection::Selection;

const BASE: &str = include_str!("tests/base.toml");
const REPORTED: &str = "over, reported only; this run is not a pull request";
const WAITING: &str = "over, not in the merge-base policy; enforced from the next change";
const TANGLE: &str = "t-tangle";
const OUTSIDE: &str = "task-0000000000000004";
const HEADS: [(&str, Option<usize>, bool); 4] = [
    ("7", Some(5), true),
    ("7", None, true),
    (ADDRESSING, Some(5), false),
    (OUTSIDE, Some(5), false),
];

#[test]
fn standing_budgets_are_enforced_from_the_merge_base_and_a_new_cap_waits() {
    let policy: Policy = toml::from_str(&format!("{}{STANDING}", Policy::DEFAULTS)).unwrap();
    let mut ledger = Ledger::standing(ADDRESSING, &policy);
    ledger.enforce_from(Some(BASE)).unwrap();
    let shown = ledger.statement(&policy).to_string();
    let blocking = ledger.blocking(&policy, &Selection::Workspace);
    assert_eq!(blocking, ["t-gone", "t-open-tasks"]);
    assert!(shown.contains(WAITING) && !shown.contains("t-extra"));
    ledger.report_only();
    let reported = ledger.statement(&policy).to_string();
    assert!(
        ledger.blocking(&policy, &Selection::Workspace).is_empty() && reported.contains(REPORTED)
    );
}

#[test]
fn an_over_standing_measure_blocks_its_division_until_a_change_moves_it_toward_its_line() {
    let policy: Policy = toml::from_str(&format!("{}{STANDING}", Policy::DEFAULTS)).unwrap();
    let unreached = Selection::Crates(BTreeSet::new());
    let blocked = HEADS.map(|(link, head, _)| {
        let mut ledger = Ledger::standing(link, &policy);
        ledger.enforce_from(Some(BASE)).unwrap();
        head.into_iter().for_each(|head| {
            ledger.spend(Measure::TangleItems, Amount::Count(Most::from(head)));
        });
        ledger.blocking(&policy, &unreached).contains(&TANGLE.to_owned())
    });
    assert_eq!(blocked, HEADS.map(|(.., blocked)| blocked));
}
