mod included_files;

use std::path::PathBuf;

use crate::reach::Reach;
use included_files::IncludedFiles;
use task::Failure;
use task::PackageGraph;
use task::SourceFile;
use task::Workspace;
use task::{BranchName, GitArgument, Program, Stdout};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Changes {
    SinceMain,
    Everything,
}

impl Changes {
    pub fn reaches(self, workspace: &Workspace, base: &BranchName) -> Result<Vec<Reach>, Failure> {
        let changed = self.paths(|| Self::since_main(workspace, base))?;
        self.reaching(workspace.packages(), &changed, || workspace.source_files())
    }

    pub fn paths(
        self,
        since_main: impl FnOnce() -> Result<Vec<PathBuf>, Failure>,
    ) -> Result<Vec<PathBuf>, Failure> {
        match self {
            Changes::Everything => Ok(Vec::new()),
            Changes::SinceMain => since_main(),
        }
    }

    pub fn reaching(
        self,
        packages: &PackageGraph,
        changed: &[PathBuf],
        sources: impl FnOnce() -> Result<Vec<SourceFile>, Failure>,
    ) -> Result<Vec<Reach>, Failure> {
        match self {
            Changes::Everything => Ok(vec![Reach::Everything]),
            Changes::SinceMain => Self::reached(packages, changed, sources),
        }
    }

    fn since_main(workspace: &Workspace, base: &BranchName) -> Result<Vec<PathBuf>, Failure> {
        Ok(workspace
            .line(Program::Git)
            .argument(GitArgument::Diff)
            .argument(GitArgument::NameOnly)
            .argument(GitArgument::NoRenames)
            .argument(GitArgument::Relative)
            .argument(GitArgument::MergeBase)
            .argument(base)
            .captured::<Stdout>()
            .map_err(|_| Failure::NoMergeBase)?
            .lines()
            .map(PathBuf::from)
            .collect())
    }

    fn reached(
        packages: &PackageGraph,
        changed: &[PathBuf],
        sources: impl FnOnce() -> Result<Vec<SourceFile>, Failure>,
    ) -> Result<Vec<Reach>, Failure> {
        let included = if changed
            .iter()
            .all(|path| Reach::reached_by(path, packages).is_some())
        {
            IncludedFiles::default()
        } else {
            IncludedFiles::collect(&sources()?)
        };
        Ok(changed
            .iter()
            .flat_map(|path| included.reach(path, packages))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::{Changes, Reach};
    use std::path::PathBuf;
    use task::PackageGraph;
    const METADATA: &str = r#"{"workspace_root": "/w", "packages": [
        {"name": "eval", "manifest_path": "/w/crates/eval/Cargo.toml", "dependencies": []}
    ]}"#;
    const KNOWN: &str = "crates/eval/src/a.rs";
    const ROWS: [(Changes, &str, Option<&str>); 3] = [
        (Changes::Everything, KNOWN, None),
        (Changes::SinceMain, KNOWN, Some("eval")),
        (Changes::SinceMain, "LICENSE", None),
    ];

    #[test]
    fn everything_lists_no_path_and_a_diff_lists_its_own() {
        let listed = |changes: Changes| changes.paths(|| Ok(vec![KNOWN.into()])).unwrap();
        let both = (listed(Changes::Everything), listed(Changes::SinceMain));
        assert_eq!(both, (vec![], vec![PathBuf::from(KNOWN)]));
    }

    #[test]
    fn everything_reaches_all_and_a_diff_reaches_its_package_or_all_for_an_unknown_path() {
        let graph = PackageGraph::from_metadata(METADATA).unwrap();
        let reach = |(changes, path, _): &(Changes, &str, Option<&str>)| {
            let sources = || Ok(Vec::new());
            changes
                .reaching(&graph, &[PathBuf::from(path)], sources)
                .unwrap()
        };
        let expected = |(_, _, package): &(Changes, &str, Option<&str>)| {
            vec![package.map_or(Reach::Everything, |name| {
                Reach::Package(name.parse().unwrap())
            })]
        };
        assert_eq!(
            ROWS.iter().map(reach).collect::<Vec<_>>(),
            ROWS.iter().map(expected).collect::<Vec<_>>()
        );
    }
}
