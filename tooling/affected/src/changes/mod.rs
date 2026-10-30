mod included_files;

use std::path::PathBuf;

use crate::reach::Reach;
use included_files::IncludedFiles;
use task::Failure;
use task::PackageGraph;
use task::SourceFile;
use task::Workspace;
use task::{BranchName, CommandLine, GitArgument, Program, Stdout, Trusted};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Changes {
    SinceMain,
    Between(Trusted, BranchName),
    Everything,
}

impl Changes {
    pub fn reaches(
        &self,
        workspace: &Workspace,
        base: &BranchName,
    ) -> Result<Vec<Vec<Reach>>, Failure> {
        let reached = |line: CommandLine| {
            let output = line
                .captured::<Stdout>()
                .map_err(|_| Failure::NoMergeBase)?;
            Self::reached(workspace.packages(), &Self::listed(&output), || {
                workspace.source_files()
            })
        };
        match self {
            Changes::Everything => Ok(vec![vec![Reach::Everything]]),
            Changes::SinceMain => reached(Self::since_main(workspace, base)).map(|set| vec![set]),
            Changes::Between(trusted, head) => trusted
                .commits()
                .iter()
                .map(|commit| reached(Self::between(workspace, commit, head)))
                .collect(),
        }
    }

    pub fn since_main_paths(
        workspace: &Workspace,
        base: &BranchName,
    ) -> Result<Vec<PathBuf>, Failure> {
        Self::since_main(workspace, base)
            .captured::<Stdout>()
            .map(|output| Self::listed(&output))
            .map_err(|_| Failure::NoMergeBase)
    }

    fn since_main(workspace: &Workspace, base: &BranchName) -> CommandLine {
        Self::diff(workspace)
            .argument(GitArgument::MergeBase)
            .argument(base)
    }

    fn between(workspace: &Workspace, trusted: &BranchName, head: &BranchName) -> CommandLine {
        Self::diff(workspace).argument(trusted).argument(head)
    }

    fn diff(workspace: &Workspace) -> CommandLine {
        workspace
            .line(Program::Git)
            .argument(GitArgument::Diff)
            .argument(GitArgument::NameOnly)
            .argument(GitArgument::NoRenames)
            .argument(GitArgument::Relative)
    }

    pub(crate) fn listed(output: &str) -> Vec<PathBuf> {
        output.lines().map(PathBuf::from).collect()
    }

    pub(crate) fn reached(
        packages: &PackageGraph,
        changed: &[PathBuf],
        files: impl FnOnce() -> Result<Vec<SourceFile>, Failure>,
    ) -> Result<Vec<Reach>, Failure> {
        let included = if changed
            .iter()
            .all(|path| Reach::reached_by(path, packages).is_some())
        {
            IncludedFiles::default()
        } else {
            IncludedFiles::collect(&files()?)
        };
        Ok(changed
            .iter()
            .flat_map(|path| included.reach(path, packages))
            .collect())
    }
}
