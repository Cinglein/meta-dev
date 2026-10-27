use settings::EmbeddedDefaults;

use super::super::tests::{ADDRESSING, STANDING};
use crate::budgets::Ledger;
use crate::policy::Policy;
use crate::selection::Selection;

const BASE: &str = include_str!("tests/base.toml");
const WAITING: &str = "over, not in the merge-base policy; enforced from the next change";

#[test]
fn standing_budgets_are_enforced_from_the_merge_base_a_new_cap_waits_and_a_loosened_or_dropped_cap_binds()
 {
    let policy: Policy = toml::from_str(&format!("{}{STANDING}", Policy::DEFAULTS)).unwrap();
    let mut ledger = Ledger::standing(ADDRESSING, &policy);
    ledger.enforce_from(Some(BASE)).unwrap();
    let shown = ledger.statement(&policy).to_string();
    assert_eq!(
        ledger.blocking(&policy, &Selection::Workspace),
        ["t-gone", "t-open-tasks"]
    );
    assert!(shown.contains(WAITING) && !shown.contains("t-extra"));
}
