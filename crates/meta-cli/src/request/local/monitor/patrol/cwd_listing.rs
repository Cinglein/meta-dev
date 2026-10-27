use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use super::super::process_row::{Pid, ProcessRow};
use super::super::unreadable::Unreadable;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CwdListing(HashMap<Pid, PathBuf>);

impl CwdListing {
    const PROCESS: &'static str = "\np";
    const NAME: char = 'n';

    pub fn placed(&self, row: ProcessRow) -> ProcessRow {
        let cwd = self.0.get(&row.pid()).cloned();
        row.placed(cwd)
    }
}

impl FromStr for CwdListing {
    type Err = Unreadable;

    fn from_str(listing: &str) -> Result<Self, Self::Err> {
        let records = format!("\n{listing}");
        let found = records.split(Self::PROCESS).filter_map(|record| {
            let pid = record.lines().next()?.parse::<Pid>().ok()?;
            let cwd = record.lines().find_map(|line| line.strip_prefix(Self::NAME))?;
            Some((pid, PathBuf::from(cwd)))
        });
        Ok(Self(found.collect()))
    }
}
