mod deadline;
mod flags;
mod plist;
mod settings;
mod tick;

use chrono::{DateTime, TimeDelta, Utc};
use meta_pull_query::{LiveQuery, LiveReply};
use meta_status::{Github, GithubFailure};

use crate::request::cli_failure::CliFailure;
use crate::request::project::{Config, HeartbeatSettings};

const START: &str = "2026-10-26T12:00:00Z";
const OFFLINE: &str = "offline";
const MAIN: &str = r#""main""#;
const REQUIRED: &str = r#"["ci"]"#;
const ORPHAN_REPLY: &str = r#"{"data":{"open":{"pullRequests":{"pageInfo":{"hasNextPage":false},"nodes":[
{"number":16,"state":"OPEN","baseRefName":"main","statusCheckRollup":null}]}}}}"#;
const NOTHING_OPEN: &str = r#"{"data":{"open":{"pullRequests":{"pageInfo":{"hasNextPage":false},"nodes":[]}}}}"#;
const SETTINGS: &str = "[heartbeat]\ninterval_seconds = 300\ntick_seconds = 60\nstale_seconds = 900";

fn at(offset: i64) -> DateTime<Utc> {
    START.parse::<DateTime<Utc>>().unwrap() + TimeDelta::seconds(offset)
}

fn offline(_: &LiveQuery) -> Github {
    Github::Unreached(GithubFailure::new(OFFLINE))
}

fn reached(reply: &str) -> Github {
    let rules = [(serde_json::from_str(MAIN).unwrap(), serde_json::from_str(REQUIRED).unwrap())];
    let pulls = serde_json::from_str::<LiveReply>(reply).unwrap().answered().unwrap();
    Github::Reached(pulls.ruled(rules.into_iter().collect()).unwrap())
}

fn orphaned(_: &LiveQuery) -> Github {
    reached(ORPHAN_REPLY)
}

fn quiet(_: &LiveQuery) -> Github {
    reached(NOTHING_OPEN)
}

fn settings(text: &str) -> Result<HeartbeatSettings, CliFailure> {
    toml::from_str::<Config>(text).unwrap().heartbeat_settings()
}
