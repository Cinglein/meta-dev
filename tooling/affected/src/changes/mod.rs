mod included_files;

use std::path::PathBuf;

use crate::reach::Reach;
use included_files::IncludedFiles;
use task::Failure;
use task::Workspace;
use task::{BranchName, GitArgument, Program, Stdout};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Changes {
    SinceMain,
    Everything,
}

impl Changes {
    pub fn reaches(self, workspace: &Workspace, base: &BranchName) -> Result<Vec<Reach>, Failure> {
        self.reaching(workspace, &self.paths(workspace, base)?)
    }

    pub fn paths(self, workspace: &Workspace, base: &BranchName) -> Result<Vec<PathBuf>, Failure> {
        match self {
            Changes::Everything => Ok(Vec::new()),
            Changes::SinceMain => Self::since_main(workspace, base),
        }
    }

    pub fn reaching(
        self,
        workspace: &Workspace,
        changed: &[PathBuf],
    ) -> Result<Vec<Reach>, Failure> {
        match self {
            Changes::Everything => Ok(vec![Reach::Everything]),
            Changes::SinceMain => Self::reached(workspace, changed),
        }
    }

    fn since_main(workspace: &Workspace, base: &BranchName) -> Result<Vec<PathBuf>, Failure> {
        Ok(workspace
            .line(Program::Git)
            .argument(GitArgument::Diff)
            .argument(GitArgument::NameOnly)
            .argument(GitArgument::Relative)
            .argument(GitArgument::MergeBase)
            .argument(base)
            .captured::<Stdout>()
            .map_err(|_| Failure::NoMergeBase)?
            .lines()
            .map(PathBuf::from)
            .collect())
    }

    fn reached(workspace: &Workspace, changed: &[PathBuf]) -> Result<Vec<Reach>, Failure> {
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
