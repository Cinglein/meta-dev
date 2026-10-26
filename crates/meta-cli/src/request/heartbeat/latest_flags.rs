use std::path::Path;

use chrono::{DateTime, Utc};
use meta_schema::DueTime;

use super::heartbeat_flag::HeartbeatFlag;
use super::heartbeat_record::HeartbeatRecord;
use crate::request::cli_failure::CliFailure;
use crate::request::output::{Listed, Output};
use crate::request::project::HeartbeatSettings;

#[derive(Debug)]
pub struct LatestFlags;

impl LatestFlags {
    pub fn answer(git_dir: &Path, settings: &HeartbeatSettings, now: DateTime<Utc>) -> Result<Output, CliFailure> {
        let Some(record) = HeartbeatRecord::read(git_dir)? else {
            return Ok(Output::Flagged(Listed::new(format!("flags:\n{}", HeartbeatFlag::NeverTicked))));
        };
        let latest = record.latest();
        let age = latest.age(now);
        let limit = settings.stale_limit();
        let stale = (age > limit).then_some(HeartbeatFlag::Stale { age, limit });
        let body = format!("{}{}", stale.map(|stale| stale.to_string()).unwrap_or_default(), latest.flags());
        let flags = Some(body)
            .filter(|body| !body.is_empty())
            .map_or_else(|| "flags: none\n".to_owned(), |body| format!("flags:\n{body}"));
        let text = Listed::new(format!("heartbeat {}, {}s ago\n{flags}", DueTime::from(latest.at()), age.as_secs()));
        let judged = stale.is_some() || latest.flags().awaiting_judgement().raised().is_some();
        Ok(if judged { Output::Flagged(text) } else { Output::Listed(text) })
    }
}
