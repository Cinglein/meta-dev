mod cwd_listing;
mod judged_rows;
mod process_table;

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};

use super::culprit::Culprit;
use super::fault::Fault;
use super::finding::Finding;
use super::identity::Identity;
use super::load_average::LoadAverage;
use super::machine::Machine;
use super::marked_groups::MarkedGroups;
use super::monitor_report::MonitorReport;
use super::monitor_state::MonitorState;
use super::ownership::Ownership;
use super::pid_list::PidList;
use super::process_row::ProcessRow;
use super::reused::Reused;
use super::signal::Signal;
use super::target::Target;
use super::watch_item::WatchItem;
use crate::request::local::local_settings::MonitorSettings;
use cwd_listing::CwdListing;
use judged_rows::JudgedRows;
use process_table::ProcessTable;

#[derive(Debug)]
pub struct Patrol<'patrol, Host> {
    settings: &'patrol MonitorSettings,
    host: &'patrol Host,
}

impl<'patrol, Host: Machine> Patrol<'patrol, Host> {
    pub fn new(settings: &'patrol MonitorSettings, host: &'patrol Host) -> Self {
        Self { settings, host }
    }

    pub fn settings(&self) -> &MonitorSettings {
        self.settings
    }

    pub fn round(&self, before: &MonitorState, now: DateTime<Utc>) -> MonitorReport {
        let load = self.host.uptime().and_then(|uptime| uptime.parse::<LoadAverage>().ok());
        self.rows().map_or_else(
            || MonitorReport::new(now, load, before.clone()).found(vec![Finding::Blind], Vec::new()),
            |rows| {
                let table = ProcessTable::new(rows, self.settings, self.host.own_pid());
                let marked = table.marked(before.marked(), now);
                let judged = table.judged(&marked);
                let (stopped, over): (Vec<Culprit>, Vec<Culprit>) = judged
                    .culprits(Ownership::Agent, self.settings)
                    .into_iter()
                    .partition(|culprit| self.settings.stops(culprit.cause()).is_some());
                let unknown = judged.culprits(Ownership::Unsure, self.settings).into_iter().map(Finding::Unknown);
                let watched = self.watched(&judged, before, now);
                let promoted = watched.iter().filter(|item| self.settings.promotes(item.ticks()).is_some());
                let promoted = promoted.cloned().map(Finding::Watched);
                let findings = self.stopped(&stopped, &marked).into_iter();
                let findings: Vec<Finding> =
                    findings.chain(over.into_iter().map(Finding::OverLimit)).chain(unknown).chain(promoted).collect();
                MonitorReport::new(now, load, MonitorState::new(marked, watched)).found(findings, judged.shown())
            },
        )
    }

    fn rows(&self) -> Option<Vec<ProcessRow>> {
        let rows: Vec<ProcessRow> = self.host.table()?.lines().filter_map(|line| line.parse().ok()).collect();
        let cwds: CwdListing =
            self.host.cwds(&PidList::new(rows.iter().map(ProcessRow::pid).collect()))?.parse().ok()?;
        Some(rows.into_iter().map(|row| cwds.placed(row)).collect())
    }

    fn watched(&self, judged: &JudgedRows, before: &MonitorState, now: DateTime<Utc>) -> Vec<WatchItem> {
        let items = judged.tracked().filter_map(|row| row.watched(self.settings, now));
        items
            .map(|item| {
                let seen = before.watched(&item).cloned();
                item.followed(seen.as_ref())
            })
            .collect()
    }

    fn stopped(&self, culprits: &[Culprit], marked: &MarkedGroups) -> Vec<Finding> {
        let known: Vec<Identity> =
            culprits.iter().filter_map(|culprit| Identity::observe(self.host, culprit.pid())).collect();
        let (live, refused) = self.confirmed(culprits.to_vec(), &known);
        let targets: BTreeSet<Target> = live.iter().map(Culprit::target).collect();
        let after = (!targets.is_empty()).then(|| {
            self.host.signal(Signal::Term, &targets);
            self.host.pause(self.settings.grace().duration());
            let rows = self.rows().unwrap_or_default();
            ProcessTable::new(rows, self.settings, self.host.own_pid()).judged(marked)
        });
        let survivors = after.unwrap_or_default().survivors(&live);
        let (doomed, refused_late) = self.confirmed(survivors, &known);
        let killed: BTreeSet<Target> = doomed.iter().map(Culprit::target).collect();
        (!killed.is_empty()).then(|| self.host.signal(Signal::Kill, &killed));
        let terminated = live.into_iter().map(|culprit| Finding::Signalled(Signal::Term, culprit));
        let killing = doomed.into_iter().map(|culprit| Finding::Signalled(Signal::Kill, culprit));
        terminated.chain(killing).chain(refused).chain(refused_late).collect()
    }

    fn confirmed(&self, culprits: Vec<Culprit>, known: &[Identity]) -> (Vec<Culprit>, Vec<Finding>) {
        let checked = culprits.into_iter().map(|culprit| {
            let seen = known.iter().find(|identity| identity.of(culprit.pid()).is_some());
            let outcome = seen.ok_or(Reused(culprit.pid())).and_then(|identity| identity.confirmed(self.host));
            (culprit, outcome)
        });
        let (kept, refused): (Vec<_>, Vec<_>) = checked.partition(|(_, outcome)| outcome.is_ok());
        let refusals = refused.into_iter().filter_map(|(_, outcome)| outcome.err());
        (
            kept.into_iter().map(|(culprit, _)| culprit).collect(),
            refusals.map(|reused| Finding::Failed(Fault::shown(reused))).collect(),
        )
    }
}
