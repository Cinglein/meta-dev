use std::fmt;
use std::str::FromStr;

use meta_schema::{Id, Lease, Task};
use meta_state::State;

use crate::request::output::Listed;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LeaseLine {
    task: Id<Task>,
    lease: Id<Lease>,
}

impl LeaseLine {
    pub const PREFIX: &'static str = "lease:";

    pub fn new(task: Id<Task>, lease: Id<Lease>) -> Self {
        Self { task, lease }
    }

    pub fn live_in(&self, state: &State) -> Result<(), Listed> {
        let live = state.leases(self.task).into_iter().find(|leased| leased.lease() == self.lease);
        live.map(|_| ()).ok_or_else(|| Listed::new(format!("{} is not a live lease on {}", self.lease, self.task)))
    }
}

impl FromStr for LeaseLine {
    type Err = Listed;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let words: Vec<&str> = line.strip_prefix(Self::PREFIX).unwrap_or_default().split_whitespace().collect();
        let unparsable = |reason: &dyn fmt::Display| Listed::new(format!("unparsable `{line}`: {reason}"));
        match words.as_slice() {
            [task, lease] => Ok(Self {
                task: task.parse().map_err(|failure| unparsable(&failure))?,
                lease: lease.parse().map_err(|failure| unparsable(&failure))?,
            }),
            _ => Err(unparsable(&"expected `lease: <task-id> <lease-id>`")),
        }
    }
}

impl fmt::Display for LeaseLine {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {} {}", Self::PREFIX, self.task, self.lease)
    }
}
