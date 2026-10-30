use meta_log::MemoryLog;

use super::{LeaseLine, Prompt};
use crate::request::cli_failure::{CliFailure, Invalid};
use crate::request::{Request, ScalePolicy};

const POLICY: &str = "[dispatch]\nissuers = [\"owner\"]\n";
const OPEN: [&str; 5] = ["meta", "open", "--division", "features-and-maintenance", "x"];
const OWNER: &str = "owner";
const AGENT: &str = "worker";
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
    let meta = Meta(MemoryLog::default(), toml::from_str(POLICY).unwrap());
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
fn only_a_dispatch_issuer_may_lease_a_task_to_an_agent() {
    let meta = Meta(MemoryLog::default(), toml::from_str(POLICY).unwrap());
    let task = meta.run(OWNER, &OPEN).split_whitespace().nth(ID_INDEX).unwrap_or_default().to_owned();
    assert!(meta.run(AGENT, &["meta", CLAIM, &task, "--agent", AGENT]).contains(UNISSUED));
}
