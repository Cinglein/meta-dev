mod after;

use std::time::Duration;

use chrono::{DateTime, Utc};
use meta_schema::Block;
use serde::Deserialize;

use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::hook::CallPhase;
use after::After;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Liveness {
    #[serde(default)]
    silent_after: Option<After>,
    #[serde(default)]
    long_call_after: Option<After>,
    #[serde(default)]
    reply_within: Option<After>,
    #[serde(default)]
    ci_run_within: Option<After>,
}

impl Liveness {
    const UNSET: &'static str =
        "tooling/policy.toml: [liveness] ci_run_within is missing; status cannot judge a CI wait without it";

    pub fn ci_run_within(&self) -> Result<Duration, CliFailure> {
        self.ci_run_within.map(After::duration).ok_or_else(|| CliFailure::raised::<Setup>(Self::UNSET))
    }

    pub fn limit(&self, call: CallPhase) -> Option<Duration> {
        let after = match call {
            CallPhase::Started => self.long_call_after,
            CallPhase::Finished => self.silent_after,
        };
        after.map(After::duration)
    }

    pub fn asked(&self, block: Block, now: DateTime<Utc>) -> Block {
        let due = block.reply_due().or_else(|| self.reply_within?.due_after(now));
        due.into_iter().fold(block, Block::reply_by)
    }
}
