mod built_binary;

use meta_golden::{Golden, Goldens, Reach};
use task::{CargoArgument, Program, Stdout, Workspace};

use crate::task::failure::Failure;
use built_binary::BuiltBinary;

pub struct GoldenRefresh;

impl GoldenRefresh {
    const META_PACKAGE: &'static str = "meta-cli";
    const META: &'static str = "meta";
    const MESSAGES: &'static str = "--message-format=json";
    const TEST_TARGET: &'static str = "--test";
    const VIEWS: [[&'static str; 2]; 1] = [["meta-git-view", "goldens"]];
    const BLESSED: &'static str = "1";
    const CI: &'static str = "CI";

    pub fn run(workspace: &Workspace, reach: Reach) -> Result<(), Failure> {
        let cargo = || workspace.line(Program::Cargo);
        let build =
            cargo().argument(CargoArgument::Build).argument(CargoArgument::Package).argument(Self::META_PACKAGE);
        let meta = BuiltBinary::located(&build.argument(Self::MESSAGES).captured::<Stdout>()?, Self::META);
        let meta = meta.ok_or(Failure::Unbuilt(Self::META))?;
        Goldens::workspace().refreshed(BuiltBinary::dir(&meta), reach)?;
        Self::VIEWS.iter().try_for_each(|[package, target]| {
            let views = cargo().argument(CargoArgument::Test).argument(CargoArgument::Package).argument(package);
            let views = views.argument(Self::TEST_TARGET).argument(target);
            let checked = match std::env::var_os(Self::CI) {
                Some(_) => views,
                None => views.environment(Golden::BLESS, Self::BLESSED),
            };
            Ok(checked.run()?)
        })
    }
}
