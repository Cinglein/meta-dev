use std::fmt;

use meta_run::Run;

use super::process_row::Pid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PidList(Vec<Pid>);

impl PidList {
    pub fn new(pids: Vec<Pid>) -> Self {
        Self(pids)
    }

    pub fn appended(&self, run: Run) -> Run {
        run.arg(self.to_string())
    }
}

impl fmt::Display for PidList {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let shown: Vec<String> = self.0.iter().map(ToString::to_string).collect();
        formatter.write_str(&shown.join(","))
    }
}
