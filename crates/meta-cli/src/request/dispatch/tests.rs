use std::path::Path;

use meta_log::MemoryLog;
use meta_schema::{Claim, Event, Id, LeaseGrant};

use super::{Brief, Dispatch, LeaseLine, Prompt};
use crate::request::cli_failure::{CliFailure, Invalid};
use crate::request::writing::Writing;
use crate::request::{Request, ScalePolicy};

const OPEN: [&str; 5] = ["meta", "open", "--division", "features-and-maintenance", "x"];
const OWNER: &str = "owner";
const AGENT: &str = "worker";
const ROOT: &str = "/some/project";
const CLAIM: &str = "claim";
const RETIRE: &str = "retire";
const ID_INDEX: usize = 4;
const TASK_WORD: usize = 1;
const LEASE_WORD: usize = 2;
const UNLEASED: &str = "work on it";
const UNPARSABLE: &str = "lease: task-0000000000000001 nonsense";
const UNISSUED: &str = "[dispatch] issuers";
const AFTER: [&[&str]; 6] = [&[], &[CLAIM], &["done"], &["release"], &[RETIRE], &["done", "reopen"]];
const ADMITTED: [bool; 10] = [true, false, true, false, false, false, false, false, false, false];

#[derive(Default)]
struct Meta(MemoryLog, ScalePolicy);

impl Meta {
    fn run(&self, actor: &str, words: &[&str]) -> String {
        let request = Request::try_from(words.iter().map(ToString::to_string).collect::<Vec<_>>());
        let actor = || actor.parse().map_err(CliFailure::raised::<Invalid>);
        let answered = request.and_then(|request| request.answer_from(&self.0, &self.1, actor));
        answered.map_or_else(|failure| failure.to_string(), |output| output.to_string())
    }

    fn step(&self, task: &str, verb: &str) -> String {
        match verb {
            CLAIM => self.run(OWNER, &["meta", CLAIM, task, "--agent", AGENT]),
            RETIRE => self.run(OWNER, &["meta", RETIRE, task, "gone"]),
            verb => self.run(AGENT, &["meta", verb, task]),
        }
    }

    fn briefed(&self, after: &[&str]) -> [String; 2] {
        let task = self.run(OWNER, &OPEN).split_whitespace().nth(ID_INDEX).unwrap_or_default().to_owned();
        let first = self.step(&task, CLAIM);
        let last = after.iter().map(|verb| self.step(&task, verb)).last().unwrap_or_else(|| first.clone());
        [first, last]
    }

    fn admitted(&self, prompt: &str) -> bool {
        serde_json::from_value::<Prompt>(serde_json::json!(prompt)).unwrap().admitted(&self.0).is_ok()
    }
}

#[test]
fn only_a_lease_line_naming_the_live_lease_of_its_open_task_admits_an_agent() {
    let meta = Meta::default();
    let [[live, _], [stale, renewed], [done, _], [released, _], [retired, _], [reopened, _]] =
        AFTER.map(|after| meta.briefed(after));
    let word = |brief: &str, at: usize| {
        let leased = brief.lines().find(|text| text.starts_with(LeaseLine::PREFIX));
        leased.and_then(|text| text.split_whitespace().nth(at)).unwrap_or_default().to_owned()
    };
    let wrong_task = [LeaseLine::PREFIX.to_owned(), word(&done, TASK_WORD), word(&live, LEASE_WORD)].join(" ");
    let prompts = [&live, &stale, &renewed, &done, &released, &retired, &reopened, &wrong_task];
    let admitted: Vec<bool> = prompts.iter().map(|prompt| meta.admitted(prompt)).collect();
    let refused = [UNLEASED, UNPARSABLE].map(|prompt| meta.admitted(prompt));
    assert_eq!([admitted, refused.to_vec()].concat(), ADMITTED, "{live}\n{stale}\n{renewed}");
}

#[test]
fn a_dispatch_is_announced_only_by_a_non_empty_actor_name() {
    let announced = [None, Some(String::new()), Some(OWNER.to_owned())].map(|variable| Dispatch::named(variable).ok());
    assert_eq!(announced, [None, None, Some(OWNER.parse().unwrap())]);
}

#[test]
fn a_rooted_dispatch_briefs_the_project_root() {
    let meta = Meta::default();
    let task = meta.run(OWNER, &OPEN).split_whitespace().nth(ID_INDEX).unwrap_or_default().to_owned();
    let dispatch = Dispatch::new(Claim::new(task.parse().unwrap(), None), AGENT.parse().unwrap());
    let brief = dispatch.rooted(Path::new(ROOT)).answer(&meta.0, OWNER.parse().unwrap()).unwrap().to_string();
    assert!(brief.contains(&format!("{}{ROOT}", Brief::PROJECT)), "{brief}");
}

#[test]
fn only_a_dispatch_issuer_may_lease_a_task_and_only_a_lease_it_issued_admits_an_agent() {
    let meta = Meta::default();
    let task = meta.run(OWNER, &OPEN).split_whitespace().nth(ID_INDEX).unwrap_or_default().to_owned();
    assert!(meta.run(AGENT, &["meta", CLAIM, &task, "--agent", AGENT]).contains(UNISSUED));
    let id = task.parse().unwrap();
    let lease = Id::generate();
    let claim = Claim::new(id, None).leased(LeaseGrant::new(lease, AGENT.parse().unwrap()));
    Writing::new(&meta.0, AGENT.parse().unwrap(), Event::TaskClaimed(claim)).entered().unwrap();
    assert!(!meta.admitted(&LeaseLine::new(id, lease).to_string()));
}
