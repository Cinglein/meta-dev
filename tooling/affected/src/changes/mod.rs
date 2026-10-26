mod included_files;

use std::path::PathBuf;

use crate::reach::Reach;
use included_files::IncludedFiles;
use task::Failure;
use task::Workspace;
use task::{GitArgument, Program, Stdout};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Changes {
    SinceMain,
    Everything,
}

impl Changes {
    const BASE: &str = "origin/main";

    pub fn reaches(self, workspace: &Workspace) -> Result<Vec<Reach>, Failure> {
        match self {
            Changes::Everything => Ok(vec![Reach::Everything]),
            Changes::SinceMain => Self::since_main(workspace),
        }
    }

    fn since_main(workspace: &Workspace) -> Result<Vec<Reach>, Failure> {
        let packages = workspace.packages();
        let changed: Vec<PathBuf> = workspace
            .line(Program::Git)
            .argument(GitArgument::Diff)
            .argument(GitArgument::NameOnly)
            .argument(GitArgument::MergeBase)
            .argument(Self::BASE)
            .captured::<Stdout>()?
            .lines()
            .map(PathBuf::from)
            .collect();
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
