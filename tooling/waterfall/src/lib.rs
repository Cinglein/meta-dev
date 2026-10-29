mod compile;
mod crate_time;
mod seconds;
mod section;
mod timed_unit;

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::iter;
use std::path::{Path, PathBuf};

use serde::de::Error;

use compile::Compile;
use crate_time::CrateTime;
use task::{CrateName, Failure, Site};
use timed_unit::TimedUnit;

#[derive(Debug)]
pub struct Waterfall {
    units: Vec<TimedUnit>,
}

impl Waterfall {
    const UNIT_DATA: &str = "const UNIT_DATA = ";

    pub fn shown(reports: impl Iterator<Item = PathBuf>) -> Result<Option<String>, Failure> {
        reports
            .map(|report| Self::read(&report).map(|waterfall| waterfall.to_string()))
            .try_fold(None, |shown: Option<String>, waterfall| Ok(Some(shown.unwrap_or_default() + &waterfall?)))
    }

    pub fn read(report: &Path) -> Result<Waterfall, Failure> {
        let site = || Site::File(report.to_path_buf());
        let html = fs::read_to_string(report).map_err(|error| Failure::Io { site: site(), error })?;
        Self::parse(&html).map_err(|error| Failure::TimingReport { site: site(), error })
    }

    fn parse(html: &str) -> Result<Waterfall, serde_json::Error> {
        let data = html
            .split_once(Self::UNIT_DATA)
            .map(|(_, data)| data)
            .ok_or_else(|| serde_json::Error::custom(Self::UNIT_DATA))?;
        serde_json::Deserializer::from_str(data)
            .into_iter::<Vec<TimedUnit>>()
            .next()
            .unwrap_or_else(|| Err(serde_json::Error::custom(Self::UNIT_DATA)))
            .map(|units| Waterfall { units })
    }

    fn last(&self) -> Option<&TimedUnit> {
        self.units.iter().max_by(|one, other| one.ended(other))
    }

    fn critical_path(&self) -> Vec<&TimedUnit> {
        let mut path: Vec<&TimedUnit> =
            iter::successors(self.last(), |unit| self.units.iter().find(|before| before.unblocked(unit))).collect();
        path.reverse();
        path
    }

    fn crates(&self) -> Vec<CrateTime<'_>> {
        let mut crates: Vec<CrateTime<'_>> = self
            .units
            .iter()
            .fold(BTreeMap::<&CrateName, Compile>::new(), |mut crates, unit| {
                let compile = crates.entry(unit.crate_name()).or_default();
                *compile = compile.plus(unit);
                crates
            })
            .into_iter()
            .map(|(name, compile)| CrateTime::new(name, compile))
            .collect();
        crates.sort_by(CrateTime::slower);
        crates
    }
}

impl fmt::Display for Waterfall {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = self.critical_path();
        let whole = |units: &[&TimedUnit]| units.iter().fold(Compile::default(), |sum, unit| sum.plus(unit));
        let all: Vec<&TimedUnit> = self.units.iter().collect();
        writeln!(
            formatter,
            "waterfall: {} units, wall {}, compile {}",
            self.units.len(),
            self.last().map(TimedUnit::end).unwrap_or_default(),
            whole(&all)
        )?;
        writeln!(formatter, "critical path: {} units, {}", path.len(), whole(&path))?;
        path.iter().try_for_each(|unit| writeln!(formatter, "  {unit}"))?;
        writeln!(formatter, "per crate:")?;
        self.crates().iter().try_for_each(|time| writeln!(formatter, "  {time}"))
    }
}

#[cfg(test)]
mod tests {
    use super::Waterfall;

    const REPORT: &str = include_str!("report.html");
    const SHOWN: &str = include_str!("shown.txt");

    #[test]
    fn a_cargo_timing_report_shows_its_critical_path_and_per_crate_times() {
        let shown = Waterfall::parse(REPORT).map(|waterfall| waterfall.to_string());
        assert_eq!(shown.ok().as_deref(), Some(SHOWN));
        assert!(Waterfall::parse(SHOWN).is_err());
    }
}
