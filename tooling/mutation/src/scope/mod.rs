mod mutants_build;
mod mutants_run;
mod output_directory;
mod test_packages;

pub use mutants_build::MutantsBuild;
pub use mutants_run::MutantsRun;

use std::env;
use std::fs;
use std::path::Path;

use affected::TestedCrates;
use affected::{CrateLoad, Packing};
use output_directory::OutputDirectory;
use task::Failure;
use task::Shard;
use task::Site;
use task::Workspace;
use task::{Branch, BranchName, Interval, Policy};
use task::{CargoArgument, CommandLine, GitArgument, MutantsArgument, Program, Stdout};
use test_packages::TestPackages;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    ChangedLines,
    Workspace,
}

impl Scope {
    const DIFF_FILE: &str = "mutants.diff";
    const SHARD_VARIABLE: &str = "MUTANTS_SHARD";

    pub fn applied(
        self,
        line: CommandLine,
        build: &MutantsBuild,
        workspace: &Workspace,
        policy: &Policy,
    ) -> Result<Vec<MutantsRun>, Failure> {
        let output = OutputDirectory::cleared(workspace.target().path())?;
        let line = line.argument(MutantsArgument::InPlace);
        match self {
            Scope::ChangedLines => {
                let shards = match env::var(Self::SHARD_VARIABLE) {
                    Ok(spelled) => {
                        Self::write_diff(
                            workspace.root(),
                            policy.branch(Branch::Base),
                            &workspace.target().path().join(Self::DIFF_FILE),
                        )?;
                        vec![spelled.parse()?]
                    }
                    Err(_) => Self::changed_shards(build, workspace, policy)?,
                };
                let line = line
                    .argument(MutantsArgument::InDiff)
                    .argument(workspace.target().path().join(Self::DIFF_FILE).display())
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

    pub fn changed_shards(build: &MutantsBuild, workspace: &Workspace, policy: &Policy) -> Result<Vec<Shard>, Failure> {
        Self::write_diff(
            workspace.root(),
            policy.branch(Branch::Base),
            &workspace.target().path().join(Self::DIFF_FILE),
        )?;
        let listing = build
            .line(CargoArgument::Mutants)
            .argument(MutantsArgument::InDiff)
            .argument(workspace.target().path().join(Self::DIFF_FILE).display());
        let shards =
            Packing::shards(&CrateLoad::listed(listing, policy, Interval::DiffJob)?, policy, Interval::DiffJob);
        let tested = TestedCrates::collect(&workspace.source_files()?);
        TestPackages::baselines(&shards, workspace.packages(), &tested, build)
            .iter()
            .try_for_each(|baseline| build.passes(baseline))?;
        Ok(shards)
    }

    fn shard_runs(
        line: &CommandLine,
        workspace: &Workspace,
        output: &OutputDirectory,
        shards: &[Shard],
    ) -> Result<Vec<MutantsRun>, Failure> {
        let tested = TestedCrates::collect(&workspace.source_files()?);
        TestPackages::each_run(shards, workspace.packages(), &tested)
            .into_iter()
            .map(|(run, tested_by)| {
                let directory = output.prepared_for(&run)?;
                let prepared = line
                    .clone()
                    .argument(MutantsArgument::Output)
                    .argument(directory.display())
                    .argument(MutantsArgument::TestWorkspaceOff);
                Ok(MutantsRun::new(tested_by.applied_to(run.applied_to(prepared)), directory))
            })
            .collect()
    }

    fn write_diff(root: &Path, base: &BranchName, file: &Path) -> Result<(), Failure> {
        let diff = CommandLine::new(Program::Git)
            .in_directory(root.to_path_buf())
            .argument(GitArgument::Diff)
            .argument(GitArgument::Relative)
            .argument(GitArgument::MergeBase)
            .argument(base)
            .captured::<Stdout>()?;
        fs::write(file, diff).map_err(|error| Failure::Io { site: Site::File(file.to_path_buf()), error })
    }
}

#[cfg(test)]
mod tests {
    use super::Scope;
    use std::{env, fs, path::Path, process, process::Command};
    const SOURCE: &str = "consumer/a.rs";
    const BASE: &str = "origin/main";
    const RELATIVE: &str = "+++ b/a.rs";
    const GIT: [&[&str]; 4] = [
        &["init", "--quiet"],
        &["add", "--all"],
        &["-c", "user.name=t", "-c", "user.email=t@invalid", "commit", "--quiet", "-m", "t"],
        &["update-ref", "refs/remotes/origin/main", "HEAD"],
    ];

    #[test]
    fn the_diff_of_a_workspace_below_the_git_top_level_names_paths_from_its_root() {
        let repository = env::temp_dir().join(format!("meta-diff-{}", process::id()));
        fs::create_dir_all(repository.join(SOURCE).parent().unwrap()).unwrap();
        fs::write(repository.join(SOURCE), BASE).unwrap();
        assert!(GIT.iter().all(|arguments| {
            Command::new("git").args(*arguments).current_dir(&repository).status().unwrap().success()
        }));
        fs::write(repository.join(SOURCE), SOURCE).unwrap();
        let file = repository.join("mutants.diff");
        let root = repository.join(Path::new(SOURCE).parent().unwrap());
        Scope::write_diff(&root, &BASE.parse().unwrap(), &file).unwrap();
        assert!(fs::read_to_string(&file).unwrap().contains(RELATIVE));
        fs::remove_dir_all(&repository).unwrap();
    }
}
