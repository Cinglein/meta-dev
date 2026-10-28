mod mutants_settings;

use std::fs;
use std::io::ErrorKind;

use mutants_settings::MutantsSettings;
use task::Failure;
use task::Site;
use task::Workspace;
use task::{CargoArgument, CommandLine, Program};

#[derive(Debug)]
pub struct MutantsBuild(CommandLine);

impl MutantsBuild {
    const SHARED: [CargoArgument; 1] = [CargoArgument::AllFeatures];

    pub fn checked(workspace: &Workspace) -> Result<MutantsBuild, Failure> {
        let path = workspace.root().join(MutantsSettings::FILE);
        let settings = match fs::read_to_string(&path) {
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(String::new()),
            read => read.map_err(|error| Failure::Io { site: Site::File(path), error }),
        }?;
        Self::sharing(workspace.line(Program::Cargo), &settings)
    }

    pub fn sharing(cargo: CommandLine, settings: &str) -> Result<MutantsBuild, Failure> {
        settings.parse::<MutantsSettings>().map(|_| MutantsBuild(cargo))
    }

    pub fn line(&self, subcommand: CargoArgument) -> CommandLine {
        self.0.clone().argument(subcommand).arguments(Self::SHARED)
    }
}
