use std::fmt;

use chrono::{DateTime, Utc};
use meta_door::MetaStore;
use meta_schema::DueTime;

use super::finding::Finding;
use super::judged_row::JudgedRow;
use super::load_average::LoadAverage;
use super::monitor_state::MonitorState;
use crate::request::cli_failure::CliFailure;
use crate::request::local::LocalFile;

#[derive(Debug)]
pub struct MonitorReport {
    taken: DateTime<Utc>,
    load: Option<LoadAverage>,
    state: MonitorState,
    obvious: Vec<Finding>,
    rows: Vec<JudgedRow>,
}

impl MonitorReport {
    const FILE: &'static str = "meta-local/monitor-report.txt";
    const NONE_YET: &'static str = "monitor: no report yet; run meta monitor tick\n";

    pub fn new(taken: DateTime<Utc>, load: Option<LoadAverage>, state: MonitorState) -> Self {
        Self { taken, load, state, obvious: Vec::new(), rows: Vec::new() }
    }

    #[must_use]
    pub fn found(self, obvious: Vec<Finding>, rows: Vec<JudgedRow>) -> Self {
        Self { obvious, rows, ..self }
    }

    pub fn obvious(&self) -> &[Finding] {
        &self.obvious
    }

    pub fn state(&self) -> &MonitorState {
        &self.state
    }

    pub fn write(&self, store: &MetaStore) -> Result<(), CliFailure> {
        LocalFile::new(store.entry(Self::FILE)).replaced(&self.to_string())
    }

    pub fn latest(store: &MetaStore) -> Result<String, CliFailure> {
        let text = LocalFile::new(store.entry(Self::FILE)).text()?;
        Ok(if text.is_empty() { Self::NONE_YET.to_owned() } else { text })
    }
}

impl fmt::Display for MonitorReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let load = self.load.map_or_else(|| "unknown".to_owned(), |load| load.to_string());
        writeln!(formatter, "monitor {}, machine load {load} (context only)", DueTime::from(self.taken))?;
        writeln!(formatter, "obvious:")?;
        self.obvious.iter().try_for_each(|finding| write!(formatter, "  {finding}"))?;
        writeln!(formatter, "watch:")?;
        write!(formatter, "{}", self.state)?;
        writeln!(formatter, "processes under this project's roots:")?;
        self.rows.iter().try_for_each(|row| write!(formatter, "{row}"))
    }
}
