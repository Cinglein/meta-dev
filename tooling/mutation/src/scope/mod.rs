mod mutants_build;
mod mutants_run;
mod output_directory;
mod test_packages;

pub use mutants_build::MutantsBuild;
pub use mutants_run::MutantsRun;

use std::env;
use std::fs;
use std::io;
use std::path::Path;

use meta_git::{Reads as _, RelativeDiff, Repository};

use affected::TestedCrates;
use affected::{CrateLoad, Packing};
use output_directory::OutputDirectory;
use task::Failure;
use task::Shard;
use task::Site;
use task::Workspace;
use task::{Branch, BranchName, Interval, Policy};
use task::{CargoArgument, CommandLine, MutantsArgument};
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
                let tested = Self::tested(workspace, &shards)?;
                let runs = Self::shard_runs(&line, &output, &tested)?.into_iter();
                Ok(runs.zip(TestPackages::baselines(&tested, build)).map(|(run, tests)| run.baselined(tests)).collect())
            }
            Scope::Workspace => match env::var(Self::SHARD_VARIABLE) {
                Ok(spelled) => Self::shard_runs(&line, &output, &Self::tested(workspace, &[spelled.parse()?])?),
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
        Ok(Packing::shards(&CrateLoad::listed(listing, policy, Interval::DiffJob)?, policy, Interval::DiffJob))
    }

    fn tested(workspace: &Workspace, shards: &[Shard]) -> Result<Vec<TestPackages>, Failure> {
        let tested = TestedCrates::collect(&workspace.source_files()?);
        Ok(TestPackages::each_run(shards, workspace.packages(), &tested))
    }

    fn shard_runs(
        line: &CommandLine,
        output: &OutputDirectory,
        runs: &[TestPackages],
    ) -> Result<Vec<MutantsRun>, Failure> {
        runs.iter()
            .map(|tested_by| {
                let directory = output.prepared_for(tested_by.run())?;
                let prepared = line
                    .clone()
                    .argument(MutantsArgument::Output)
                    .argument(directory.display())
                    .argument(MutantsArgument::TestWorkspaceOff);
                Ok(MutantsRun::new(tested_by.applied_to(prepared), directory))
            })
            .collect()
    }

    fn write_diff(root: &Path, base: &BranchName, file: &Path) -> Result<(), Failure> {
        let unread = |error| Failure::Io { site: Site::File(root.to_path_buf()), error };
        let repository = Repository::discovered(root).map_err(|failure| unread(io::Error::other(failure)))?;
        let repository = repository.ok_or_else(|| unread(io::Error::from(io::ErrorKind::NotFound)))?;
        let asked = RelativeDiff::new(base.as_ref().to_string_lossy().into_owned(), root.to_path_buf());
        let diff = repository.read(&asked).map_err(|failure| unread(io::Error::other(failure)))?;
        fs::write(file, diff).map_err(|error| Failure::Io { site: Site::File(file.to_path_buf()), error })
    }
}

#[cfg(test)]
mod tests {
    use super::Scope;
    use std::fs;
    const SOURCE: &str = "consumer/a.rs";
    const BASE: &str = "origin/main";
    const TRACKING: &str = "refs/remotes/origin/main";
    const EVERY: &str = "*";
    const DIFF: &str = "mutants.diff";
    const RELATIVE: &str = "--- a/a.rs\n+++ b/a.rs\n@@ -1 +1 @@\n-origin/main\n\\ No newline at end of file\n+consumer/a.rs\n\\ No newline at end of file\n";

    #[test]
    fn the_diff_of_a_workspace_below_the_git_top_level_names_paths_from_its_root() {
        let repository = tempfile::tempdir().unwrap();
        let source = repository.path().join(SOURCE);
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(&source, BASE).unwrap();
        let git = git2::Repository::init(repository.path()).unwrap();
        let mut index = git.index().unwrap();
        index.add_all([EVERY], git2::IndexAddOption::DEFAULT, None).unwrap();
        index.write().unwrap();
        let tree = git.find_tree(index.write_tree().unwrap()).unwrap();
        let signature = git2::Signature::now(BASE, BASE).unwrap();
        let commit = git.commit(None, &signature, &signature, BASE, &tree, &[]).unwrap();
        git.reference(TRACKING, commit, true, BASE).and(git.set_head_detached(commit)).unwrap();
        fs::write(&source, SOURCE).unwrap();
        let file = repository.path().join(DIFF);
        Scope::write_diff(source.parent().unwrap(), &BASE.parse().unwrap(), &file).unwrap();
        assert!(fs::read_to_string(&file).unwrap().ends_with(RELATIVE));
    }
}
