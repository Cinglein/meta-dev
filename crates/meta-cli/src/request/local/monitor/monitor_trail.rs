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

    pub fn appended(
        &self,
        at: DateTime<Utc>,
        findings: &[Finding],
        settings: &MonitorSettings,
    ) -> Result<(), CliFailure> {
        let entries = self.entries()?;
        let recent: Vec<&MonitorEntry> =
            entries.iter().filter(|entry| settings.shows(entry.age(at)).is_some()).collect();
        let fresh =
            findings.iter().filter(|finding| recent.iter().all(|entry| entry.finding().repeats(finding).is_none()));
        let fresh: Vec<MonitorEntry> = fresh.map(|finding| MonitorEntry::new(at, finding.clone())).collect();
        if fresh.is_empty() {
            Ok(())
        } else {
            self.rewritten(&settings.kept().newest(entries.into_iter().chain(fresh).collect()))
        }
    }

    fn rewritten(&self, kept: &[MonitorEntry]) -> Result<(), CliFailure> {
        let lines: Result<Vec<String>, serde_json::Error> = kept.iter().map(serde_json::to_string).collect();
        self.0.replaced(&(lines.map_err(|failure| self.0.failure(failure))?.join("\n") + "\n"))
    }

    pub fn since(&self, settings: &MonitorSettings, now: DateTime<Utc>) -> Result<Vec<MonitorEntry>, CliFailure> {
        Ok(self.entries()?.into_iter().filter(|entry| settings.shows(entry.age(now)).is_some()).collect())
    }

    fn entries(&self) -> Result<Vec<MonitorEntry>, CliFailure> {
        let text = self.0.text()?;
        Ok(text.lines().filter_map(|line| serde_json::from_str::<MonitorEntry>(line).ok()).collect())
    }
}
