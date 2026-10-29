mod included_files;

use std::path::PathBuf;

use crate::reach::Reach;
use crate::trusted::Trusted;
use included_files::IncludedFiles;
use task::Failure;
use task::Workspace;
use task::{BranchName, CommandLine, GitArgument, Program, Stdout};

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
        match self {
            Changes::Everything => Ok(vec![vec![Reach::Everything]]),
            Changes::SinceMain => Self::reached(workspace, Self::since_main(workspace, base)?)
                .map(|reaches| vec![reaches]),
            Changes::Between(trusted, head) => trusted
                .commits()
                .iter()
                .map(|commit| Self::reached(workspace, Self::between(workspace, commit, head)?))
                .collect(),
        }
    }

    fn since_main(workspace: &Workspace, base: &BranchName) -> Result<Vec<PathBuf>, Failure> {
        Self::paths(
            workspace
                .line(Program::Git)
                .argument(GitArgument::Diff)
                .argument(GitArgument::NameOnly)
                .argument(GitArgument::Relative)
                .argument(GitArgument::MergeBase)
                .argument(base),
        )
    }

    fn between(
        workspace: &Workspace,
        trusted: &BranchName,
        head: &BranchName,
    ) -> Result<Vec<PathBuf>, Failure> {
        Self::paths(
            workspace
                .line(Program::Git)
                .argument(GitArgument::Diff)
                .argument(GitArgument::NameOnly)
                .argument(GitArgument::Relative)
                .argument(trusted)
                .argument(head),
        )
    }

    fn paths(line: CommandLine) -> Result<Vec<PathBuf>, Failure> {
        Ok(line
            .captured::<Stdout>()
            .map_err(|_| Failure::NoMergeBase)?
            .lines()
            .map(PathBuf::from)
            .collect())
    }

    fn reached(workspace: &Workspace, changed: Vec<PathBuf>) -> Result<Vec<Reach>, Failure> {
        let packages = workspace.packages();
        let included = if changed
            .iter()
            .all(|path| Reach::reached_by(path, packages).is_some())
        {
            IncludedFiles::default()
        } else {
            IncludedFiles::collect(&workspace.source_files()?)
        };
        Ok(changed
            .iter()
            .flat_map(|path| included.reach(path, packages))
            .collect())
    }
}
