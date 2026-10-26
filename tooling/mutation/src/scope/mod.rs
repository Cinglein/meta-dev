mod mutants_run;
mod output_directory;
mod test_packages;

pub use mutants_run::MutantsRun;

use std::env;
use std::fs;

use affected::TestedCrates;
use affected::{CrateLoad, Packing};
use output_directory::OutputDirectory;
use task::Failure;
use task::Shard;
use task::Site;
use task::Workspace;
use task::{Branch, Interval, Policy};
use task::{CargoArgument, CommandLine, GitArgument, MutantsArgument, Program, Stdout};
use test_packages::TestPackages;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    ChangedLines,
    Workspace,
}

impl Scope {
    const DIFF_FILE: &str = "target/mutants.diff";
    const SHARD_VARIABLE: &str = "MUTANTS_SHARD";

    pub fn applied(
        self,
        line: CommandLine,
        workspace: &Workspace,
        policy: &Policy,
    ) -> Result<Vec<MutantsRun>, Failure> {
        let output = OutputDirectory::cleared(workspace.root())?;
        let line = line.argument(MutantsArgument::InPlace);
        match self {
            Scope::ChangedLines => {
                let shards = match env::var(Self::SHARD_VARIABLE) {
                    Ok(spelled) => {
                        Self::write_diff(workspace, policy)?;
                        vec![spelled.parse()?]
                    }
                    Err(_) => Self::changed_shards(workspace, policy)?,
                };
                let line = line
                    .argument(MutantsArgument::InDiff)
                    .argument(Self::DIFF_FILE)
                    .argument(MutantsArgument::BaselineSkip);
                Self::shard_runs(&line, workspace, &output, &shards)
            }
            Scope::Workspace => match env::var(Self::SHARD_VARIABLE) {
                Ok(spelled) => Self::shard_runs(&line, workspace, &output, &[spelled.parse()?]),
                Err(_) => Ok(vec![MutantsRun::new(
                    line.argument(MutantsArgument::Output)
                        .argument(output.root().display())
                        .argument(MutantsArgument::TestWorkspaceOn),
                    output.root().to_path_buf(),
                )]),
            },
        }
    }

    pub fn changed_shards(workspace: &Workspace, policy: &Policy) -> Result<Vec<Shard>, Failure> {
        Self::write_diff(workspace, policy)?;
        let listing = workspace
            .line(Program::Cargo)
            .argument(CargoArgument::Mutants)
            .argument(MutantsArgument::InDiff)
            .argument(Self::DIFF_FILE);
        Ok(Packing::shards(&CrateLoad::listed(listing, policy)?, policy.interval(Interval::PerSlice)))
    }

    fn shard_runs(
        line: &CommandLine,
        workspace: &Workspace,
        output: &OutputDirectory,
        shards: &[Shard],
    ) -> Result<Vec<MutantsRun>, Failure> {
        let tested = TestedCrates::collect(&workspace.source_files()?);
        shards
            .iter()
            .flat_map(Shard::runs)
            .map(|run| {
                let directory = output.prepared_for(&run)?;
                let prepared = line
                    .clone()
                    .argument(MutantsArgument::Output)
                    .argument(directory.display())
                    .argument(MutantsArgument::TestWorkspaceOff);
                let tested_by = TestPackages::around(&run, workspace.packages(), &tested);
                Ok(MutantsRun::new(tested_by.applied_to(run.applied_to(prepared)), directory))
            })
            .collect()
    }

    fn write_diff(workspace: &Workspace, policy: &Policy) -> Result<(), Failure> {
        let diff = workspace
            .line(Program::Git)
            .argument(GitArgument::Diff)
            .argument(GitArgument::MergeBase)
            .argument(policy.branch(Branch::Base))
            .captured::<Stdout>()?;
        fs::write(workspace.root().join(Self::DIFF_FILE), diff)
            .map_err(|error| Failure::Io { site: Site::File(Self::DIFF_FILE.into()), error })
    }
}
