mod monitor_entry;

use std::path::Path;

use chrono::{DateTime, Utc};

use super::finding::Finding;
use crate::request::cli_failure::CliFailure;
use crate::request::local::LocalFile;
use crate::request::local::local_settings::MonitorSettings;
use monitor_entry::MonitorEntry;

#[derive(Debug)]
pub struct MonitorTrail(LocalFile);

impl MonitorTrail {
    const FILE: &'static str = "meta-local/monitor.jsonl";

    pub fn new(git_dir: &Path) -> Self {
        Self(LocalFile::new(git_dir.join(Self::FILE)))
    }

    pub fn appended(&self, at: DateTime<Utc>, findings: &[Finding]) -> Result<(), CliFailure> {
        findings.iter().try_for_each(|finding| {
            let entry = MonitorEntry::new(at, finding.clone());
            self.0.appended(serde_json::to_string(&entry).map_err(|failure| self.0.failure(failure))?)
        })
    }

    pub fn since(&self, settings: &MonitorSettings, now: DateTime<Utc>) -> Result<Vec<MonitorEntry>, CliFailure> {
        let text = self.0.text()?;
        let entries = text.lines().filter_map(|line| serde_json::from_str::<MonitorEntry>(line).ok());
        Ok(entries.filter(|entry| settings.shows(entry.age(now)).is_some()).collect())
    }
}
