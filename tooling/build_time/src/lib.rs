use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};

use task::{Amount, CargoArgument, CommandLine, Failure, Ledger, Measure, Millis, Policy, Program, Site, Workspace};

#[derive(Debug)]
pub struct BuildTime {
    root: PathBuf,
    line: CommandLine,
}

impl BuildTime {
    pub const DIRECTORY: &str = "build-time";
    const TARGET: &str = "CARGO_TARGET_DIR";

    pub fn run(workspace: &Workspace, policy: &Policy, ledger: &mut Ledger) -> Result<(), Failure> {
        let target = Some(workspace.target().path().join(Self::DIRECTORY))
            .filter(|target| !target.exists())
            .ok_or_else(|| Failure::EarlierBuild(workspace.target().path().join(Self::DIRECTORY)))?;
        let build = BuildTime {
            root: workspace.root().to_path_buf(),
            line: workspace
                .line(Program::Cargo)
                .arguments([CargoArgument::Build, CargoArgument::Workspace, CargoArgument::AllTargets])
                .arguments([CargoArgument::Timings, CargoArgument::Jobs])
                .argument(policy.compiler_tasks())
                .environment(Self::TARGET, target),
        };
        policy.budgets().builds().into_iter().try_for_each(|measure| build.timed(measure, ledger))
    }

    fn timed(&self, measure: Measure, ledger: &mut Ledger) -> Result<(), Failure> {
        measure.build_start().and_then(|start| start.touched()).map_or(Ok(()), |file| self.touch(file))?;
        let began = Instant::now();
        self.line.run()?;
        let spent = began.elapsed();
        println!("{measure} {spent:.2?}");
        ledger.spend(measure, Amount::Duration(Millis::rounded_up(spent)));
        Ok(())
    }

    fn touch(&self, file: &Path) -> Result<(), Failure> {
        let path = self.root.join(file);
        File::options()
            .append(true)
            .open(&path)
            .and_then(|opened| opened.set_modified(SystemTime::now()))
            .map_err(|error| Failure::Io { site: Site::File(path), error })
    }
}
