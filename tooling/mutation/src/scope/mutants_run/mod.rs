mod memory_watch;
mod missed_list;
mod mutants_exit;
mod uncaught;

pub use memory_watch::MemoryWatch;

use std::path::PathBuf;

use super::MutantsBuild;
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
    baseline: Option<CommandLine>,
}

impl MutantsRun {
    pub fn new(line: CommandLine, output: PathBuf) -> MutantsRun {
        MutantsRun { line, output, baseline: None }
    }

    pub fn baselined(self, baseline: Option<CommandLine>) -> MutantsRun {
        MutantsRun { baseline, ..self }
    }

    pub fn line(&self) -> &CommandLine {
        &self.line
    }

    pub fn accepted(&self, build: &MutantsBuild, budget: AvailableGib, policy: &Policy) -> Result<bool, Failure> {
        self.baseline.iter().try_for_each(|tests| build.passes(tests, budget, policy))?;
        let mut child = self.line.spawn()?;
        let status = MemoryWatch::over(&child, budget, policy).guard(&mut child, None)?;
        Ok(MutantsExit::judged(status) && MissedList::none_in(&self.output)? && MutantsExit::accepts(status))
    }
}
