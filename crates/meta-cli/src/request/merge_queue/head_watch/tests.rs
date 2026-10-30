use std::time::Duration;

use meta_golden::Golden;
use meta_pull_query::{LiveData, LiveReply};
use meta_pulls::{BaseBranch, Patience, Stack, Trunk};
use meta_state::State;

use super::HeadWatch;
use super::stack_walk::StackWalk;
use super::turn::Turn;
use crate::request::project::Project;

const LINKED: &str = include_str!("../../../../../../goldens/queue/linked.json");
const STACKED: &str = include_str!("../../../../../../goldens/queue/stacked.json");
const STACK_BASES: [&str; 3] = ["task-s301", "task-s302", "task-s303"];
const WALKS: [(&str, &[[&str; 2]]); 8] = [
    ("302", &[["301", "301"], ["302", "302"]]),
    ("304", &[["301", "301"], ["302", "302"], ["303", "303"], ["304", "304"]]),
    ("301", &[["305", "305"], ["301", "301"]]),
    ("302", &[["301", "999"], ["302", "302"]]),
    ("301", &[["306", "306"], ["301", "301"]]),
    ("301", &[["308", "308"], ["301", "301"]]),
    ("305", &[["305", "305"], ["301", "301"]]),
    ("303", &[["301", "301"], ["302", "302"], ["303", "303"]]),
];
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

struct Walked;

impl Walked {
    fn live() -> LiveData {
        let reply = format!(r#"{{"data":{{"open":{OPEN},"linked":{STACKED}}}}}"#);
        let pulls = serde_json::from_str::<LiveReply>(&reply).unwrap().answered().unwrap();
        let unruled = STACK_BASES.map(|base| (base.parse::<BaseBranch>().unwrap(), "[]"));
        let rules = [(MAIN.parse::<BaseBranch>().unwrap(), REQUIRED)].into_iter().chain(unruled);
        let rules = rules.map(|(base, required)| (base, serde_json::from_str(required).unwrap()));
        pulls.ruled(rules.collect()).unwrap()
    }

    fn stack(members: &[[&str; 2]]) -> Stack {
        let listed = members.iter().map(|[number, tail]| {
            serde_json::json!({"number": number.parse::<u64>().unwrap(), "state": "open", "head": {"sha": format!("{tail:0>40}")}})
        });
        serde_json::from_value(
            serde_json::json!({"base": {"ref": MAIN}, "open": true, "pull_requests": listed.collect::<Vec<_>>()}),
        )
        .unwrap()
    }

    fn walks(trunk: &Trunk) -> Vec<String> {
        let live = Self::live();
        let patience = Patience::new(Duration::ZERO, Duration::ZERO);
        let walk = |(target, members): (&str, &[[&str; 2]])| {
            let stack = Self::stack(members);
            format!(
                "{target}: {:?}",
                StackWalk::new(&live, &stack, trunk, target.parse().unwrap()).transition(patience)
            )
        };
        WALKS.map(walk).to_vec()
    }
}

#[test]
fn a_stack_walk_merges_the_target_only_when_every_open_member_below_is_green_and_unmoved() {
    let trunk = Trunk::new(MAIN.parse().unwrap(), "true".parse().unwrap());
    Golden::for_test(module_path!(), "walks").debugged(&Walked::walks(&trunk)).unwrap();
}
