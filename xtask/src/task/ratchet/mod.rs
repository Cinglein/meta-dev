mod baseline;
mod drift;
mod raised;
mod violation_count;

pub use baseline::Baseline;
pub use drift::Drift;
pub use raised::Raised;
pub use violation_count::ViolationCount;

use std::collections::BTreeMap;
use std::fmt;

use task::Report;

#[derive(Debug)]
pub struct Ratchet {
    drifts: Vec<Drift>,
    reports: BTreeMap<&'static str, Report>,
}

impl Ratchet {
    pub fn new(drifts: Vec<Drift>, reports: BTreeMap<&'static str, Report>) -> Ratchet {
        Ratchet { drifts, reports }
    }

    fn describe(&self, drift: &Drift, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{drift}")?;
        if drift.grew() {
            self.reports.get(drift.lint()).iter().try_for_each(|report| write!(formatter, "{report}"))
        } else {
            writeln!(formatter, "  fewer than baseline: lower it with `cargo xtask baseline`")
        }
    }
}

impl fmt::Display for Ratchet {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.drifts.iter().try_for_each(|drift| self.describe(drift, formatter))
    }
}
