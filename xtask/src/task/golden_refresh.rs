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
    const VIEWS: [&'static [&'static str]; 5] = [
        &["meta-git-view", "--test", "goldens"],
        &["meta-cli", "--lib"],
        &["meta-xtask", "--lib"],
        &["task", "--lib"],
        &["elegance", "--lib"],
    ];
    const BLESSED: &'static str = "1";
    const CI: &'static str = "CI";

    pub fn run(workspace: &Workspace, reach: Reach) -> Result<(), Failure> {
        let cargo = || workspace.line(Program::Cargo);
        let build =
            cargo().argument(CargoArgument::Build).argument(CargoArgument::Package).argument(Self::META_PACKAGE);
        let meta = BuiltBinary::located(&build.argument(Self::MESSAGES).captured::<Stdout>()?, Self::META);
        let meta = meta.ok_or(Failure::Unbuilt(Self::META))?;
        Goldens::workspace().refreshed(BuiltBinary::dir(&meta), reach)?;
        Self::VIEWS.iter().try_for_each(|words| {
            let views = cargo().argument(CargoArgument::Test).argument(CargoArgument::Package);
            let views = words.iter().fold(views, task::CommandLine::argument);
            let checked = match std::env::var_os(Self::CI) {
                Some(_) => views,
                None => views.environment(Golden::BLESS, Self::BLESSED),
            };
            Ok(checked.run()?)
        })
    }
}
