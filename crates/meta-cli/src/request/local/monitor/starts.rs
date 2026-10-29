mod identity;

use std::collections::BTreeMap;

use super::process_row::{Pid, ProcessRow};
use super::start_time::StartTime;
use identity::Identity;

#[derive(Clone, Debug)]
pub struct Starts(BTreeMap<Pid, StartTime>);

impl Starts {
    pub fn listed(rows: &[ProcessRow]) -> Self {
        Self(rows.iter().map(|row| (row.pid(), row.start_time().clone())).collect())
    }

    pub fn identity(&self, pid: Pid) -> Identity {
        Identity::new(pid, self.0.get(&pid).cloned())
    }
}
