mod after;

use std::time::Duration;

use chrono::{DateTime, Utc};
use meta_schema::Block;
use serde::Deserialize;

use after::After;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Liveness {
    #[serde(default)]
    silent_after: Option<After>,
    #[serde(default)]
    reply_within: Option<After>,
}

impl Liveness {
    pub fn silent_after(&self) -> Option<Duration> {
        self.silent_after.map(After::duration)
    }

    pub fn asked(&self, block: Block, now: DateTime<Utc>) -> Block {
        let due = block.reply_due().or_else(|| self.reply_within?.due_after(now));
        due.into_iter().fold(block, Block::reply_by)
    }
}
