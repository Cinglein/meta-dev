mod mutants_settings;

use std::fs;
use std::io::ErrorKind;
use std::time::Instant;

use super::mutants_run::MemoryWatch;
use mutants_settings::MutantsSettings;
use task::AvailableGib;
use task::FailedRuns;
use task::Failure;
use task::Site;
use task::Workspace;
use task::{CargoArgument, CommandLine, MutantsArgument, Program};
use task::{Interval, Millis, Policy};

#[derive(Debug)]
pub struct MutantsBuild {
    cargo: CommandLine,
    test_timeout: Millis,
    build_limit: Millis,
}

impl MutantsBuild {
    const SHARED: [CargoArgument; 1] = [CargoArgument::AllFeatures];

    pub fn checked(workspace: &Workspace, policy: &Policy) -> Result<MutantsBuild, Failure> {
        let path = workspace.root().join(MutantsSettings::FILE);
        let settings = match fs::read_to_string(&path) {
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(String::new()),
            read => read.map_err(|error| Failure::Io { site: Site::File(path), error }),
        }?;
        let [test_timeout, build_limit] =
            [Interval::TestTimeout, Interval::DiffJob].map(|limit| policy.interval(limit));
        Self::sharing(workspace.line(Program::Cargo), &settings, test_timeout, build_limit)
    }

    pub fn sharing(
        cargo: CommandLine,
        settings: &str,
        test_timeout: Millis,
        build_limit: Millis,
    ) -> Result<MutantsBuild, Failure> {
        settings.parse::<MutantsSettings>().map(|_| MutantsBuild { cargo, test_timeout, build_limit })
    }

    pub fn line(&self, subcommand: CargoArgument) -> CommandLine {
        self.cargo.clone().argument(subcommand).arguments(Self::SHARED)
    }

    pub fn mutating(&self) -> CommandLine {
        self.line(CargoArgument::Mutants)
            .argument(MutantsArgument::Timeout)
            .argument(self.test_timeout.duration().as_secs_f64())
    }

    pub fn passes(&self, tests: &CommandLine, budget: AvailableGib, policy: &Policy) -> Result<(), Failure> {
        let built = tests.clone().argument(CargoArgument::NoRun);
        [(&built, self.build_limit), (tests, self.test_timeout)].into_iter().try_for_each(|(line, limit)| {
            let mut child = line.spawn()?;
            let deadline = Instant::now() + limit.duration();
            let status = MemoryWatch::over(&child, budget, policy).guard(&mut child, Some(deadline))?;
            status.success().then_some(()).ok_or_else(|| Failure::Runs(FailedRuns::from(vec![line.clone()])))
        })
    }
}
