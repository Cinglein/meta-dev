use std::collections::BTreeSet;

use chrono::{DateTime, Utc};

use super::super::group_sighting::GroupSighting;
use super::super::judged_row::JudgedRow;
use super::super::marked_groups::MarkedGroups;
use super::super::ownership::Ownership;
use super::super::process_row::{GroupId, Pid, ProcessRow};
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

    pub fn marked(&self, before: &MarkedGroups, now: DateTime<Utc>) -> MarkedGroups {
        let placed = self.rows.iter().filter(|row| row.root(self.settings).is_some());
        let groups: BTreeSet<GroupId> = placed.map(ProcessRow::group).collect();
        let sightings: Vec<GroupSighting> = groups.into_iter().map(|group| self.sighting(group, now)).collect();
        before.renewed(&sightings, now, self.settings)
    }

    fn sighting(&self, group: GroupId, now: DateTime<Utc>) -> GroupSighting {
        let members: Vec<&ProcessRow> = self.rows.iter().filter(|row| row.group() == group).collect();
        let started = members.iter().map(|row| row.started(now)).min().unwrap_or(now);
        let leader = members.iter().find(|row| row.pid() == group.leader()).map(|row| row.started(now));
        let wrapped = members.iter().any(|row| self.wrapped(row).is_some()).then_some(());
        GroupSighting::new(group, started, leader, wrapped)
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
        let near = near.filter(|()| !self.shielded.contains(&row.pid()));
        match (near, self.spared_above(row), marked, placed, row.orphaned()) {
            (None, ..) | (Some(()), Some(_), None, ..) | (Some(()), None, None, _, None) => Ownership::Outside,
            (Some(()), Some(_), Some(()), ..) => Ownership::Attended,
            (Some(()), None, Some(()), Some(()), _) => Ownership::Agent,
            (Some(()), None, Some(()), None, _) | (Some(()), None, None, _, Some(())) => Ownership::Unsure,
        }
    }
}
