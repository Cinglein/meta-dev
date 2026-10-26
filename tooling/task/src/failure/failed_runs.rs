use std::fmt;

use crate::command_line::CommandLine;

#[derive(Debug)]
pub struct FailedRuns(Vec<CommandLine>);

impl From<Vec<CommandLine>> for FailedRuns {
    fn from(runs: Vec<CommandLine>) -> FailedRuns {
        FailedRuns(runs)
    }
}

impl fmt::Display for FailedRuns {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lines: Vec<String> = self.0.iter().map(|line| format!("{line} failed")).collect();
        formatter.write_str(&lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::FailedRuns;
    use crate::command_line::{CargoArgument, CommandLine, Program};
    const SHOWN: &str = "git fmt failed\ngit clippy failed";
    #[test]
    fn every_failed_run_is_named_on_its_own_line() {
        let run = |argument| CommandLine::new(Program::Git).argument(argument);
        let runs = vec![run(CargoArgument::Fmt), run(CargoArgument::Clippy)];
        assert_eq!(FailedRuns::from(runs).to_string(), SHOWN);
    }
}
