use meta_golden::{Golden, Goldens};
use task::{CargoArgument, Program, Workspace};

use crate::task::failure::Failure;

pub struct GoldenRefresh;

impl GoldenRefresh {
    const META: &'static str = "meta-cli";
    const BIN: &'static str = "target/debug";
    const TESTS_ONLY: &'static str = "--";
    const VIEWS: &'static str = "golden_view";
    const BLESSED: &'static str = "1";

    pub fn run(workspace: &Workspace) -> Result<(), Failure> {
        let cargo = || workspace.line(Program::Cargo);
        cargo().argument(CargoArgument::Build).argument(CargoArgument::Package).argument(Self::META).run()?;
        Goldens::workspace().refreshed(&workspace.root().join(Self::BIN))?;
        let views = cargo().argument(CargoArgument::Test).argument(CargoArgument::Workspace);
        Ok(views.argument(Self::TESTS_ONLY).argument(Self::VIEWS).environment(Golden::BLESS, Self::BLESSED).run()?)
    }
}
