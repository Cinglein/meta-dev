use meta_log::MemoryLog;

use super::Prompt;
use crate::request::cli_failure::{CliFailure, Invalid};
use crate::request::{Request, ScalePolicy};

const OPEN: [&str; 5] = ["meta", "open", "--division", "features-and-maintenance", "x"];
const AGENT: &str = "worker";
const ID_INDEX: usize = 4;
const UNLEASED: &str = "work on it";
const UNPARSABLE: &str = "lease: task-0000000000000001 nonsense";
const ENDINGS: [&str; 3] = ["claim", "done", "release"];
const ADMITTED: [bool; 5] = [false, true, false, false, false];

struct Meta(MemoryLog);

impl Meta {
    fn run(&self, words: &[&str]) -> String {
        let request = Request::try_from(words.iter().map(ToString::to_string).collect::<Vec<_>>());
        let actor = || AGENT.parse().map_err(CliFailure::raised::<Invalid>);
        let answered = request.and_then(|request| request.answer_from(&self.0, &ScalePolicy::default(), actor));
        answered.map_or_else(|failure| failure.to_string(), |output| output.to_string())
    }

    fn admitted(&self, prompt: &str) -> bool {
        serde_json::from_value::<Prompt>(serde_json::json!(prompt)).unwrap().admitted(&self.0).is_ok()
    }
}

#[test]
fn an_agent_starts_only_on_a_lease_line_naming_a_live_lease_on_an_open_task() {
    let meta = Meta(MemoryLog::default());
    let briefs = ENDINGS.map(|ending| {
        let task = meta.run(&OPEN).split_whitespace().nth(ID_INDEX).unwrap_or_default().to_owned();
        let brief = meta.run(&["meta", "claim", &task, "--agent", AGENT]);
        meta.run(&["meta", ending, &task]);
        brief
    });
    let [live, done, released] = briefs.each_ref().map(String::as_str);
    let prompts = [UNLEASED, live, done, released, UNPARSABLE];
    assert_eq!(prompts.map(|prompt| meta.admitted(prompt)), ADMITTED, "{briefs:?}");
}
