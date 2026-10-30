use std::time::Duration;

use meta_golden::Golden;
use meta_pull_query::{LiveData, LiveReply};
use meta_pulls::{Patience, Trunk};
use meta_state::State;

use super::HeadWatch;
use super::turn::Turn;
use crate::request::project::Project;

const LINKED: &str = include_str!("../../../../../../goldens/queue/linked.json");
const OPEN: &str = r#"{"pullRequests":{"pageInfo":{"hasNextPage":false},"nodes":[]}}"#;
const BASE: &str = r#""main""#;
const REQUIRED: &str = r#"["ci-gate"]"#;

const UNREACHABLE: &str = "[ci]\nprogram = \"/nonexistent/gh\"\nqueue_checks_millis = 1\n";
const TAKEN: [&str; 4] = ["105", "218", "215", "187"];
const LANDED: [&str; 2] = ["196", "105"];
const MAIN: &str = "main";
const CONFLICTED: &str = "105";

struct Shell;

impl Shell {
    fn live() -> LiveData {
        let reply = format!(r#"{{"data":{{"open":{OPEN},"linked":{LINKED}}}}}"#);
        let pulls = serde_json::from_str::<LiveReply>(&reply).unwrap().answered().unwrap();
        let rules = [(serde_json::from_str(BASE).unwrap(), serde_json::from_str(REQUIRED).unwrap())];
        pulls.ruled(rules.into_iter().collect()).unwrap()
    }

    fn turns(project: &Project, trunk: &Trunk) -> Vec<String> {
        let watch = |pull: &str| HeadWatch::new(project, trunk, Duration::ZERO, pull.parse().unwrap());
        let patience = Patience::new(Duration::ZERO, Duration::ZERO);
        let taken = TAKEN.map(|pull| {
            let watch = watch(pull);
            format!("{:?}", Turn::new(&watch, State::default(), Self::live()).taken(patience))
        });
        let landed = LANDED.map(|pull| {
            let watch = watch(pull);
            format!("{:?}", Turn::new(&watch, State::default(), Self::live()).landed(patience))
        });
        let watched = format!("{:?}", watch(CONFLICTED).watched());
        taken.into_iter().chain(landed).chain([watched]).collect()
    }
}

#[test]
fn each_turn_acts_skips_stops_or_closes_out_and_a_shell_failure_ends_the_head() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join(".git")).unwrap();
    std::fs::write(dir.path().join("meta.toml"), UNREACHABLE).unwrap();
    let project = Project::try_from(dir.path()).unwrap();
    let trunk = Trunk::new(MAIN.parse().unwrap(), "true".parse().unwrap());
    Golden::for_test(module_path!(), "turns").debugged(&Shell::turns(&project, &trunk)).unwrap();
}
