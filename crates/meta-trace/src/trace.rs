use std::collections::BTreeSet;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::chrome_event::ChromeEvent;
use crate::command_word::CommandWord;
use crate::field_text::FieldText;
use crate::placement::Placement;
use crate::retention::Retention;
use crate::trace_file_name::TraceFileName;

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trace {
    #[serde(rename = "traceEvents")]
    events: Vec<ChromeEvent>,
}

impl Trace {
    const TOP_LEN: usize = 5;
    const INDENT: &'static str = "  ";

    #[must_use]
    pub fn new(events: Vec<ChromeEvent>) -> Self {
        Self { events }
    }

    pub fn kept(&self, dir: &Path, word: &CommandWord, retention: Retention) -> io::Result<PathBuf> {
        let path = self.stamped(dir, word, Utc::now())?;
        retention.pruned(&Self::listed(dir)?)?;
        Ok(path)
    }

    pub fn latest(dir: &Path) -> io::Result<Option<Self>> {
        let latest = Self::listed(dir)?.pop();
        latest.map(|path| Ok(serde_json::from_slice(&std::fs::read(path)?)?)).transpose()
    }

    fn stamped(&self, dir: &Path, word: &CommandWord, at: DateTime<Utc>) -> io::Result<PathBuf> {
        std::fs::create_dir_all(dir)?;
        let path = word.file_in(dir, at);
        std::fs::write(&path, serde_json::to_vec(self)?)?;
        Ok(path)
    }

    fn listed(dir: &Path) -> io::Result<Vec<PathBuf>> {
        let absent = |failure: io::Error| (failure.kind() == io::ErrorKind::NotFound).then_some(None).ok_or(failure);
        let entries = std::fs::read_dir(dir).map(Some).or_else(absent)?.into_iter().flatten();
        let traces = entries.map(|entry| entry.map(|entry| entry.path())).filter(|path| {
            path.as_ref().map_or(true, |path| path.extension() == Some(TraceFileName::EXTENSION.as_ref()))
        });
        Ok(traces.collect::<io::Result<BTreeSet<PathBuf>>>()?.into_iter().collect())
    }

    fn critical_path(&self, outer: &ChromeEvent, indent: &str) -> Vec<String> {
        let held = |inner: &&ChromeEvent| outer.placing(inner) == Placement::Inside;
        let children: Vec<&ChromeEvent> = self.events.iter().filter(held).collect();
        let direct: Vec<&ChromeEvent> = children
            .iter()
            .copied()
            .filter(|inner| !children.iter().any(|middle| middle.placing(inner) == Placement::Inside))
            .collect();
        let before = |later: &&ChromeEvent| {
            let earlier = direct.iter().copied().filter(|event| later.placing(event) == Placement::Before);
            earlier.max_by_key(|event| event.end())
        };
        let last = direct.iter().copied().max_by_key(|event| event.end());
        let chain: Vec<&ChromeEvent> = std::iter::successors(last, before).take(direct.len()).collect();
        let deeper = format!("{indent}{}", Self::INDENT);
        let steps = chain.into_iter().rev().flat_map(|step| self.critical_path(step, &deeper));
        [format!("{indent}{outer}\n")].into_iter().chain(steps).collect()
    }

    fn counts<'trace>(&'trace self, heading: &str, counted: impl Fn(&ChromeEvent) -> bool) -> Vec<String> {
        let spawned = |event: &'trace ChromeEvent| event.spawned().filter(|_| counted(event));
        let programs: BTreeSet<&FieldText> = self.events.iter().filter_map(spawned).collect();
        let tally =
            |program: &FieldText| self.events.iter().filter_map(spawned).filter(|seen| *seen == program).count();
        programs.into_iter().map(|program| format!("{heading} {program} {}\n", tally(program))).collect()
    }
}

impl fmt::Display for Trace {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut longest: Vec<&ChromeEvent> = self.events.iter().collect();
        longest.sort_by_key(|event| std::cmp::Reverse(event.duration()));
        let path = longest.first().map(|root| self.critical_path(root, Self::INDENT)).unwrap_or_default();
        writeln!(formatter, "critical path:\n{}top {} spans:", path.concat(), Self::TOP_LEN)?;
        longest.iter().take(Self::TOP_LEN).try_for_each(|event| writeln!(formatter, "{}{event}", Self::INDENT))?;
        let spawns = self.counts("spawns", |_| true);
        let network = self.counts("network", |event| event.args().networked());
        formatter.write_str(&[spawns, network].concat().concat())
    }
}
