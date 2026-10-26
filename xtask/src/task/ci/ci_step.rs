use std::time::Instant;

use strum::{Display, EnumIter};

use crate::task::clippy::Clippy;
use crate::task::failure::Failure;
use crate::task::lint::Lint;
use crate::task::test::Test;
use meta_lints::Project;
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
}

impl CiStep {
    pub fn timed(
        self,
        workspace: &Workspace,
        selection: &Selection,
        policy: &Policy,
        project: &Project,
    ) -> Result<(), Failure> {
        let started = Instant::now();
        let outcome = match self {
            CiStep::Fmt => Ok(workspace
                .line(Program::Cargo)
                .argument(CargoArgument::Fmt)
                .argument(CargoArgument::All)
                .argument(CargoArgument::Check)
                .run()?),
            CiStep::Clippy => Ok(Clippy::run(workspace, selection, policy)?),
            CiStep::Test => Ok(Test::run(workspace, selection, policy)?),
            CiStep::TestTime => Ok(TestTime::run(workspace, selection, policy)?),
            CiStep::Lint => Lint::run(workspace, policy, project),
        };
        println!("ci: {self} {:.1?}", started.elapsed());
        outcome
    }
}
