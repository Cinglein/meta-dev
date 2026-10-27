use meta_schema::Record;
use meta_state::State;
use settings::EmbeddedDefaults;

use super::{ChangeLink, Standing};
use crate::budgets::{Amount, Ledger};
use crate::policy::{Most, Policy};

const LOG: &str = include_str!("tests/severity.jsonl");
const CAPS: &str = include_str!("tests/severity.toml");
const COUNTED: &str = "sev-";
const SPENT: [(&str, usize); 6] = [
    ("sev-cost-critical", 1),
    ("sev-critical", 1),
    ("sev-major", 2),
    ("sev-minor", 1),
    ("sev-nav-lost", 1),
    ("sev-unrated", 1),
];

#[test]
fn a_severity_cap_counts_open_items_rolled_up_to_its_division_at_its_level_or_unrated() {
    let policy: Policy = toml::from_str(&format!("{}{CAPS}", Policy::DEFAULTS)).unwrap();
    let records: Vec<Record> = LOG
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let mut ledger = Ledger::default();
    ledger.count(Some(Standing::read(
        &State::fold(&records),
        &ChangeLink::default(),
    )));
    let spent = policy
        .budgets()
        .0
        .iter()
        .map(|(id, budget)| (id.to_string(), ledger.spend_of(budget)));
    let counted = spent.filter(|(id, _)| id.starts_with(COUNTED));
    let expected = SPENT.map(|(id, count)| (id.to_owned(), Some(Amount::Count(Most::from(count)))));
    assert_eq!(counted.collect::<Vec<_>>(), expected);
}
