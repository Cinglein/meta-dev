use std::collections::BTreeMap;
use std::time::Duration;

use meta_golden::Golden;

use crate::{LivePull, Patience, RequiredChecks, RuledPull};

const LINKED: &str = include_str!("../../../goldens/queue/linked.json");
const REQUIRED: &str = r#"["ci-gate"]"#;
const LIMIT: Duration = Duration::from_secs(60);

struct Recorded;

impl Recorded {
    fn steps(waited: Duration) -> String {
        let pulls: BTreeMap<String, LivePull> = serde_json::from_str(LINKED).unwrap();
        let required: RequiredChecks = serde_json::from_str(REQUIRED).unwrap();
        let patience = Patience::new(waited, LIMIT);
        let stepped = pulls.values().filter_map(|pull| RuledPull::new(pull, Some(&required))).map(|ruled| {
            let head = ruled.head();
            format!("{head}: {}\n", head.clone().step(patience))
        });
        stepped.collect()
    }
}

#[test]
fn each_recorded_head_steps_to_its_one_exit() {
    Golden::for_test(module_path!(), "steps").shown(&Recorded::steps(Duration::ZERO)).unwrap();
}

#[test]
fn a_head_still_checking_after_the_limit_stops_with_checks_timeout() {
    Golden::for_test(module_path!(), "timeout").shown(&Recorded::steps(LIMIT + LIMIT)).unwrap();
}
