use std::collections::BTreeSet;

use super::super::judged_row::JudgedRow;
use super::super::marked_groups::MarkedGroups;
use super::super::ownership::Ownership;
use super::super::process_row::{Pid, ProcessRow};
use super::judged_rows::JudgedRows;
use crate::request::local::local_settings::MonitorSettings;

#[derive(Debug)]
pub struct ProcessTable<'settings> {
    rows: Vec<ProcessRow>,
    settings: &'settings MonitorSettings,
    spared: BTreeSet<Pid>,
    shielded: BTreeSet<Pid>,
    wrappers: BTreeSet<Pid>,
}

impl<'settings> ProcessTable<'settings> {
    pub fn new(rows: Vec<ProcessRow>, settings: &'settings MonitorSettings, own: Pid) -> Self {
        let spared: BTreeSet<Pid> = rows.iter().filter_map(|row| row.spared_pid(settings)).chain([own]).collect();
        let shielded = spared.iter().flat_map(|pid| pid.ancestry(&rows)).collect();
        let wrappers = rows.iter().filter_map(ProcessRow::wrapper_pid).collect();
        Self { rows, settings, spared, shielded, wrappers }
    }

    pub fn marked(&self, before: &MarkedGroups) -> MarkedGroups {
        let live = self.rows.iter().map(ProcessRow::group).collect();
        let wrapped = self.rows.iter().filter(|row| self.wrapped(row).is_some()).map(ProcessRow::group);
        before.renewed(&live, wrapped.collect())
    }

    pub fn judged(&self, marked: &MarkedGroups) -> JudgedRows {
        JudgedRows::new(
            self.rows
                .iter()
                .map(|row| {
                    let marked = self.wrapped(row).or_else(|| marked.holds(row.group()));
                    let root = row.root(self.settings).cloned();
                    let near = root.as_ref().map(|_| ()).or_else(|| row.named(self.settings));
                    JudgedRow::new(row.clone(), self.ownership(row, marked, near), marked, root)
                })
                .collect(),
        )
    }

    fn wrapped(&self, row: &ProcessRow) -> Option<()> {
        row.pid().ancestry(&self.rows).iter().any(|pid| self.wrappers.contains(pid)).then_some(())
    }

    fn spared_above(&self, row: &ProcessRow) -> Option<Pid> {
        row.pid().ancestry(&self.rows).into_iter().skip(1).find(|pid| self.spared.contains(pid))
    }

    fn ownership(&self, row: &ProcessRow, marked: Option<()>, near: Option<()>) -> Ownership {
        let placed = row.root(self.settings).map(|_| ());
        let near = near.filter(|()| !self.shielded.contains(&row.pid()) || placed.is_some());
        match (near, self.spared_above(row).filter(|_| placed.is_none()), marked.or(placed), placed, row.orphaned()) {
            (None, ..) | (Some(()), Some(_), None, ..) | (Some(()), None, None, _, None) => Ownership::Outside,
            (Some(()), Some(_), Some(()), ..) => Ownership::Attended,
            (Some(()), None, Some(()), Some(()), _) => Ownership::Agent,
            (Some(()), None, Some(()), None, _) | (Some(()), None, None, _, Some(())) => Ownership::Unsure,
        }
    }
}
