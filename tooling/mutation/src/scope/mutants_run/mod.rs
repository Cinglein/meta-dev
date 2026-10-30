mod memory_watch;
mod missed_list;
mod mutants_exit;

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
        MutantsExit::judged(status).map_or(Ok(false), |exit| {
            let uncaught = MissedList::uncaught_in(&self.output)?;
            eprint!("{}", uncaught.iter().map(|list| list.to_string() + "\n").collect::<String>());
            Ok(uncaught.is_empty() && exit.clean())
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::scope::{MutantsBuild, MutantsRun};
    use settings::EmbeddedDefaults;
    use task::{CommandLine, Interval, MemoryShare, Policy, Program};
    const SH: &str = "sh";
    const SCRIPT: &str = "-c";
    const EMPTY: &str = "";
    const FILES: [&str; 2] = ["mutants.out/missed.txt", "mutants.out/timeout.txt"];
    const TIMED_OUT: &str = "a.rs:1:1: delete x";
    const EXITS: [(&str, bool); 3] = [("exit 0", true), ("exit 3", false), ("exit 2", false)];

    #[test]
    fn only_a_clean_exit_with_empty_lists_passes_and_a_listed_timeout_fails_even_on_a_clean_exit() {
        let policy = Policy::defaults();
        let output = std::env::temp_dir().join(format!("meta-run-{}", std::process::id()));
        let [missed, timed_out] = FILES.map(|file| output.join(file));
        std::fs::create_dir_all(missed.parent().unwrap()).unwrap();
        let written = [&missed, &timed_out].into_iter().all(|file| std::fs::write(file, EMPTY).is_ok());
        let sh = CommandLine::new(Program::Executable(SH.into())).argument(SCRIPT);
        let limit = policy.interval(Interval::DiscoverEvery);
        let build = MutantsBuild::sharing(sh.clone(), EMPTY, limit, limit).unwrap();
        let run = |script| MutantsRun::new(sh.clone().argument(script), output.clone());
        let accepted = |script| run(script).accepted(&build, policy.memory(MemoryShare::Granted), &policy).unwrap();
        let judged = EXITS.map(|(script, _)| accepted(script));
        std::fs::write(&timed_out, TIMED_OUT).unwrap();
        let [(clean, _), ..] = EXITS;
        let listed = accepted(clean);
        std::fs::remove_dir_all(&output).unwrap();
        assert!(written && judged == EXITS.map(|(_, verdict)| verdict) && !listed);
    }
}
