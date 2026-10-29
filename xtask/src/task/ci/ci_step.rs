use std::time::{Duration, Instant};

use strum::{Display, EnumIter, EnumString};

use crate::task::clippy::Clippy;
use crate::task::failure::Failure;
use crate::task::lint::Lint;
use crate::task::supply::Supply;
use task::Ledger;
use task::Policy;
use task::Selection;
use task::SpecSettings;
use task::Workspace;
use task::{CargoArgument, Program};
use test_time::TestTime;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display, EnumIter, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum CiStep {
    Fmt,
    Clippy,
    Test,
    Lint,
    Spec,
    Supply,
}

impl CiStep {
    pub fn timed(
        self,
        workspace: &Workspace,
        selection: &Selection,
        policy: &Policy,
        ledger: &mut Ledger,
    ) -> Result<Duration, Failure> {
        let started = Instant::now();
        let outcome = match self {
            CiStep::Fmt => workspace
                .line(Program::Cargo)
                .argument(CargoArgument::Fmt)
                .argument(CargoArgument::All)
                .argument(CargoArgument::Check)
                .run()
                .map_err(Failure::from),
            CiStep::Clippy => Clippy::run(workspace, selection, policy).map_err(Failure::from),
            CiStep::Test => TestTime::run(workspace, selection, policy, ledger).map_err(Failure::from),
            CiStep::Lint => workspace
                .source_files()
                .map_err(Failure::from)
                .and_then(|files| Lint::run(workspace.root(), &files, policy, ledger)),
            CiStep::Spec => policy.spec().map_or(Ok(()), |spec| {
                workspace
                    .line(Program::Cargo)
                    .argument(CargoArgument::Run)
                    .argument(CargoArgument::Package)
                    .argument(spec.package())
                    .argument(CargoArgument::ToolArguments)
                    .argument(SpecSettings::CHECK)
                    .run()
                    .map_err(Failure::from)
            }),
            CiStep::Supply => Supply::deny(workspace, policy)
                .and_then(|()| Supply::semver(workspace, selection, policy))
                .and_then(|()| Supply::secrets(workspace)),
        };
        let elapsed = started.elapsed();
        println!("ci: {self} {elapsed:.1?}");
        outcome.map(|()| elapsed)
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::CiStep;
    use crate::task::Task;
    use test_time::TestRunner;

    const STEPS: [&str; 6] = ["fmt", "clippy", "test", "lint", "spec", "supply"];

    #[test]
    fn the_suite_runs_once_in_the_one_step_that_also_times_it_through_the_runner_task() {
        assert!(CiStep::iter().map(|step| step.to_string()).eq(STEPS));
        assert_eq!(TestRunner::TASK.parse::<Task>(), Ok(Task::TestRunner));
    }
}
