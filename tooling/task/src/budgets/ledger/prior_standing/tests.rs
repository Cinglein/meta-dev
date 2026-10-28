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
const UNMEASURED: &str = "over, unmeasured at the merge base or head; fails closed";
const ENDS: [(&str, Option<usize>, Option<usize>, Selection); 9] = [
    ("7", Some(6), Some(5), Selection::Workspace),
    ("7", Some(5), Some(5), Selection::Workspace),
    ("7", Some(5), Some(5), Selection::Crates(BTreeSet::new())),
    ("7", Some(5), Some(6), Selection::Crates(BTreeSet::new())),
    ("7", Some(5), Some(4), Selection::Workspace),
    ("7", None, Some(5), Selection::Crates(BTreeSet::new())),
    ("7", Some(5), None, Selection::Crates(BTreeSet::new())),
    (ADDRESSING, Some(5), Some(6), Selection::Workspace),
    (OUTSIDE, Some(5), Some(6), Selection::Workspace),
];
const BLOCKED: [bool; 9] = [false, true, false, true, false, true, true, false, false];
const UNMEASURED_ENDS: usize = 2;

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
    let spent = |ledger: Ledger, tangle: Option<usize>| {
        tangle.into_iter().fold(ledger, |mut ledger, tangle| {
            ledger.spend(Measure::TangleItems, Amount::Count(Most::from(tangle)));
            ledger
        })
    };
    let checked = ENDS.map(|(link, base, head, selection)| {
        let mut ledger = spent(Ledger::standing(link, &policy), head);
        ledger.enforce_from(Some(BASE)).unwrap();
        ledger.measured_at_base(spent(Ledger::default(), base));
        let shown = ledger.statement(&policy).to_string();
        let tangled = ledger.blocking(&policy, &selection);
        (
            tangled.contains(&TANGLE.to_owned()),
            shown.contains(UNMEASURED),
        )
    });
    assert_eq!(checked.map(|(blocked, _)| blocked), BLOCKED);
    let noted = checked.iter().filter(|(_, noted)| *noted).count();
    assert_eq!(noted, UNMEASURED_ENDS);
}
