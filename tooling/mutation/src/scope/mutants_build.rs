mod mutants_settings;

use std::fs;
use std::io::ErrorKind;

use mutants_settings::MutantsSettings;
use task::Failure;
use task::Site;
use task::Workspace;
use task::{CargoArgument, CommandLine, MutantsArgument, Program, Stdout};
use task::{Interval, Millis, Policy};

#[derive(Debug)]
pub struct MutantsBuild {
    cargo: CommandLine,
    test_timeout: Millis,
}

impl MutantsBuild {
    const SHARED: [CargoArgument; 1] = [CargoArgument::AllFeatures];

    pub fn checked(workspace: &Workspace, policy: &Policy) -> Result<MutantsBuild, Failure> {
        let path = workspace.root().join(MutantsSettings::FILE);
        let settings = match fs::read_to_string(&path) {
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(String::new()),
            read => read.map_err(|error| Failure::Io { site: Site::File(path), error }),
        }?;
        Self::sharing(workspace.line(Program::Cargo), &settings, policy.interval(Interval::TestTimeout))
    }

    pub fn sharing(cargo: CommandLine, settings: &str, test_timeout: Millis) -> Result<MutantsBuild, Failure> {
        settings.parse::<MutantsSettings>().map(|_| MutantsBuild { cargo, test_timeout })
    }

    pub fn line(&self, subcommand: CargoArgument) -> CommandLine {
        self.cargo.clone().argument(subcommand).arguments(Self::SHARED)
    }

    pub fn mutating(&self) -> CommandLine {
        self.line(CargoArgument::Mutants)
            .argument(MutantsArgument::Timeout)
            .argument(self.test_timeout.duration().as_secs_f64())
    }

    pub fn passes(&self, tests: &CommandLine) -> Result<(), Failure> {
        let printed = tests.clone().argument(CargoArgument::NoRun).captured::<Stdout>()?;
        eprint!("{printed}");
        tests.stdout_within(self.test_timeout.duration()).map(|printed| eprint!("{printed}"))
    }
}
