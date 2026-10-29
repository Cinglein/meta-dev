use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use meta_golden::Golden;
use meta_run::{CommandLine, Port, PullArgv};
use meta_schema::PullRequest;

use crate::{Action, Departure, LivePull, Patience, RequiredChecks, RuledPull, Stop, TransitionVisitor, Trunk};

const LINKED: &str = include_str!("../../../goldens/queue/linked.json");
const REQUIRED: &str = r#"["ci-gate"]"#;
const LIMIT: Duration = Duration::from_secs(60);
const STRICT: &str = "true";

struct Argv;

impl TransitionVisitor for Argv {
    type Output = String;

    fn act(self, action: Action) -> String {
        let run = |call| CommandLine::from(&Port::<PullArgv>::new(Path::new("")).run(call, action.operands()));
        action.call().map(|call| format!(" => {}", run(call))).unwrap_or_default()
    }

    fn conflict(self, _: PullRequest) -> String {
        String::new()
    }

    fn stop(self, _: Stop) -> String {
        String::new()
    }

    fn left(self, _: Departure) -> String {
        String::new()
    }
}

struct Recorded;

impl Recorded {
    fn steps(default: &str, strict: &str, waited: Duration) -> String {
        let pulls: BTreeMap<String, LivePull> = serde_json::from_str(LINKED).unwrap();
        let required: RequiredChecks = serde_json::from_str(REQUIRED).unwrap();
        let trunk = Trunk::new(default.parse().unwrap(), strict.parse().unwrap());
        let patience = Patience::new(waited, LIMIT);
        let stepped = pulls.values().filter_map(|pull| RuledPull::new(pull, Some(&required))).map(|ruled| {
            let transition = ruled.head(&trunk).step(patience);
            format!("{transition}{}\n", transition.clone().visit(Argv))
        });
        stepped.collect()
    }
}

#[test]
fn each_recorded_head_steps_to_its_one_exit_and_a_merge_names_its_tested_head() {
    Golden::for_test(module_path!(), "steps").shown(&Recorded::steps("main", STRICT, Duration::ZERO)).unwrap();
}

#[test]
fn a_head_still_checking_after_the_limit_stops_with_checks_timeout() {
    Golden::for_test(module_path!(), "timeout").shown(&Recorded::steps("main", STRICT, LIMIT + LIMIT)).unwrap();
}

#[test]
fn an_open_head_off_the_default_branch_or_under_loose_rules_stops() {
    let barred = [Recorded::steps("trunk", STRICT, Duration::ZERO), Recorded::steps("main", "false", Duration::ZERO)];
    Golden::for_test(module_path!(), "barred").shown(&barred.concat()).unwrap();
}
