mod cwd_listing;

use std::collections::BTreeSet;

use super::culprit::Culprit;
use super::finding::Finding;
use super::judged_row::JudgedRow;
use super::load_average::LoadAverage;
use super::machine::Machine;
use super::ownership::Ownership;
use super::pid_list::PidList;
use super::process_row::ProcessRow;
use super::signal::Signal;
use super::target::Target;
use crate::request::local::local_settings::{CoreShare, MonitorSettings};
use cwd_listing::CwdListing;

#[derive(Debug)]
pub struct Patrol<'patrol, Host> {
    settings: &'patrol MonitorSettings,
    host: &'patrol Host,
}

impl<'patrol, Host: Machine> Patrol<'patrol, Host> {
    pub fn new(settings: &'patrol MonitorSettings, host: &'patrol Host) -> Self {
        Self { settings, host }
    }

    pub fn findings(&self) -> Vec<Finding> {
        self.judged().map_or_else(
            || vec![Finding::Blind],
            |judged| {
                let culprits = Self::culprits(&judged, Ownership::Agent, self.settings);
                let unsure = Self::culprits(&judged, Ownership::Unsure, self.settings).into_iter().map(Finding::Unsure);
                let agents = judged.iter().filter_map(|row| row.owned(Ownership::Agent));
                let seen: CoreShare = agents.map(ProcessRow::cpu).sum();
                let cpu = self.settings.agent_cpu().passed_by(seen).map(|limit| Finding::AgentCpu { seen, limit });
                self.stopped(&culprits).into_iter().chain(unsure).chain(self.load()).chain(cpu).collect()
            },
        )
    }

    fn judged(&self) -> Option<Vec<JudgedRow>> {
        let rows: Vec<ProcessRow> = self.host.table()?.lines().filter_map(|line| line.parse().ok()).collect();
        let pids = PidList::new(rows.iter().map(ProcessRow::pid).collect());
        let cwds: CwdListing = self.host.cwds(&pids)?.parse().ok()?;
        let rows: Vec<ProcessRow> = rows.into_iter().map(|row| cwds.placed(row)).collect();
        let guarded = self.host.own_pid().ancestry(&rows);
        Some(rows.into_iter().map(|row| row.judged(self.settings, &guarded)).collect())
    }

    fn culprits(judged: &[JudgedRow], wanted: Ownership, settings: &MonitorSettings) -> Vec<Culprit> {
        let whole_group = |row: &ProcessRow| {
            let agent = |other: &JudgedRow| other.owned(Ownership::Agent).is_some() && wanted == Ownership::Agent;
            judged.iter().all(|other| other.row().grouped_with(row).is_none() || agent(other)).then_some(())
        };
        let owned = judged.iter().filter_map(|row| row.owned(wanted));
        owned.filter_map(|row| row.cause(settings).map(|cause| row.culprit(cause, whole_group(row)))).collect()
    }

    fn stopped(&self, culprits: &[Culprit]) -> Vec<Finding> {
        let targets: BTreeSet<Target> = culprits.iter().map(Culprit::target).collect();
        let after = (!targets.is_empty()).then(|| {
            self.host.signal(Signal::Term, &targets);
            self.host.pause(self.settings.grace().duration());
            self.judged().unwrap_or_default()
        });
        let alive: Vec<&ProcessRow> = after.iter().flatten().filter_map(|row| row.owned(Ownership::Agent)).collect();
        let survivors: Vec<&Culprit> =
            culprits.iter().filter(|culprit| alive.iter().any(|row| row.survives(culprit).is_some())).collect();
        let killed: BTreeSet<Target> = survivors.iter().map(|culprit| culprit.target()).collect();
        (!killed.is_empty()).then(|| self.host.signal(Signal::Kill, &killed));
        let terminated = culprits.iter().map(|culprit| Finding::Signalled(Signal::Term, culprit.clone()));
        terminated
            .chain(survivors.into_iter().map(|culprit| Finding::Signalled(Signal::Kill, culprit.clone())))
            .collect()
    }

    fn load(&self) -> Option<Finding> {
        let seen = self.host.uptime()?.parse::<LoadAverage>().ok()?.cores();
        self.settings.overloaded(seen, self.host.cores()).map(|limit| Finding::Load { seen, limit })
    }
}
