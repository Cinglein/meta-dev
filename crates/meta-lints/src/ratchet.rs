use std::fmt;

use crate::drift::Drift;
use crate::rule::Findings;

#[derive(Clone, Debug)]
pub struct Ratchet {
    drifts: Vec<Drift>,
    findings: Findings,
}

impl Ratchet {
    pub fn new(drifts: Vec<Drift>, findings: Findings) -> Ratchet {
        Ratchet { drifts, findings }
    }

    fn describe(&self, drift: &Drift, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{drift}")?;
        if drift.grew() {
            self.findings
                .headline(drift.rule())
                .into_iter()
                .try_for_each(|headline| writeln!(formatter, "  {headline}"))?;
            self.findings
                .violations_in(drift.rule(), drift.file())
                .into_iter()
                .try_for_each(|violation| writeln!(formatter, "  {violation}"))
        } else {
            writeln!(formatter, "  fewer than baseline: lower it with `baseline`")
        }
    }
}

impl fmt::Display for Ratchet {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.drifts.iter().try_for_each(|drift| self.describe(drift, formatter))
    }
}
