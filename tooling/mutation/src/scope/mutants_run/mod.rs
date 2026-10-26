mod memory_watch;
mod missed_list;
mod mutants_exit;

use std::path::PathBuf;

use memory_watch::MemoryWatch;
use missed_list::MissedList;
use mutants_exit::MutantsExit;
use task::AvailableGib;
use task::CommandLine;
use task::Failure;
use task::Policy;

#[derive(Debug)]
pub struct MutantsRun {
    line: CommandLine,
    output: PathBuf,
}

impl MutantsRun {
    pub fn new(line: CommandLine, output: PathBuf) -> MutantsRun {
        MutantsRun { line, output }
    }

    pub fn line(&self) -> &CommandLine {
        &self.line
    }

    pub fn accepted(&self, budget: AvailableGib, policy: &Policy) -> Result<bool, Failure> {
        let mut child = self.line.spawn()?;
        let status = MemoryWatch::over(&child, budget, policy).guard(&mut child)?;
        Ok(MutantsExit::accepts(status) && MissedList::in_output(&self.output)?.is_empty())
    }
}
