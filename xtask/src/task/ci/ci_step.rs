use std::time::Instant;

use strum::{Display, EnumIter};

use crate::task::clippy::Clippy;
use crate::task::failure::Failure;
use crate::task::lint::Lint;
use crate::task::test::Test;
use task::Policy;
use task::Selection;
use task::Workspace;
use task::{CargoArgument, Program};
use test_time::TestTime;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display, EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum CiStep {
    Fmt,
    Clippy,
    Test,
    TestTime,
    Lint,
    Spec,
}

impl CiStep {
    pub fn timed(self, workspace: &Workspace, selection: &Selection, policy: &Policy) -> Result<(), Failure> {
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
            CiStep::Test => Test::run(workspace, selection, policy).map_err(Failure::from),
            CiStep::TestTime => TestTime::run(workspace, selection, policy).map_err(Failure::from),
            CiStep::Lint => workspace
                .source_files()
                .map_err(Failure::from)
                .and_then(|files| Lint::run(workspace.root(), &files, policy)),
            CiStep::Spec => policy.spec().map_or(Ok(()), |spec| {
                workspace
                    .line(Program::Cargo)
                    .argument(CargoArgument::Test)
                    .argument(CargoArgument::Package)
                    .argument(spec.package())
                    .run()
                    .map_err(Failure::from)
            }),
        };
        println!("ci: {self} {:.1?}", started.elapsed());
        outcome
    }
}
