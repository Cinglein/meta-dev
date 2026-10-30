mod changes;
mod crate_load;
mod packing;
mod reach;
mod tested_crates;
#[cfg(test)]
mod trusted;

pub use changes::Changes;
pub use crate_load::CrateLoad;
pub use packing::Packing;
pub use tested_crates::TestedCrates;

use reach::Reach;
use std::path::PathBuf;
use task::BranchName;
use task::Failure;
use task::Footprint;
use task::PackageGraph;
use task::Policy;
use task::Selection;
use task::SourceFile;
use task::Workspace;

#[derive(Debug)]
pub struct Affected;

impl Affected {
    pub fn selection(
        workspace: &Workspace,
        changes: &Changes,
        base: &BranchName,
    ) -> Result<Selection, Failure> {
        Ok(Self::tested(
            workspace.packages(),
            &changes.reaches(workspace, base)?,
        ))
    }

    pub fn footprint(
        workspace: &Workspace,
        base: &BranchName,
        policy: &Policy,
    ) -> Result<Footprint, Failure> {
        let paths = Changes::since_main_paths(workspace, base)?;
        Self::footprint_of(
            workspace.packages(),
            paths,
            || workspace.source_files(),
            policy,
        )
    }

    fn footprint_of(
        packages: &PackageGraph,
        paths: Vec<PathBuf>,
        files: impl FnOnce() -> Result<Vec<SourceFile>, Failure>,
        policy: &Policy,
    ) -> Result<Footprint, Failure> {
        let reaches = Changes::reached(packages, &paths, files)?;
        Ok(Footprint::new(
            Self::reached(packages, &reaches),
            paths.into_iter().collect(),
            policy,
        ))
    }

    fn tested(graph: &PackageGraph, trusted: &[Vec<Reach>]) -> Selection {
        trusted
            .iter()
            .map(|reaches| Self::reached(graph, reaches))
            .fold(Selection::Workspace, |tested, demanded| {
                tested.meet(&demanded)
            })
    }

    fn reached(graph: &PackageGraph, reaches: &[Reach]) -> Selection {
        if reaches.contains(&Reach::Everything) {
            Selection::Workspace
        } else {
            Selection::Crates(
                graph.dependents_closure(reaches.iter().filter_map(Reach::package).collect()),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Affected, Footprint, Reach, Selection};
    use settings::EmbeddedDefaults;
    use std::collections::BTreeSet;
    use std::path::PathBuf;
    use task::{PackageGraph, Policy};
    const METADATA: &str = r#"{"workspace_root": "/w", "packages": [
        {"name": "board", "manifest_path": "/w/crates/board/Cargo.toml", "dependencies": []},
        {"name": "eval", "manifest_path": "/w/crates/eval/Cargo.toml", "dependencies": [{"name": "board", "path": "/w/crates/board"}]},
        {"name": "lint", "manifest_path": "/w/tooling/lint/Cargo.toml", "dependencies": []}
    ]}"#;
    const BOARD: &str = "board";
    const EVAL: &str = "eval";
    const NOTHING: Reach = Reach::Nothing;
    const EVERYTHING: Reach = Reach::Everything;

    #[test]
    fn a_footprint_holds_the_dependents_closure_and_the_paths_and_the_workspace_for_everything() {
        let graph = PackageGraph::from_metadata(METADATA).unwrap();
        let (policy, path) = (Policy::defaults(), PathBuf::from("crates/board/a.rs"));
        let made = |path: &PathBuf| {
            Affected::footprint_of(&graph, vec![path.clone()], || Ok(Vec::new()), &policy).unwrap()
        };
        let want = |path: &PathBuf, reach: Selection| {
            Footprint::new(reach, BTreeSet::from([path.clone()]), &policy)
        };
        let both = BTreeSet::from([BOARD.parse().unwrap(), EVAL.parse().unwrap()]);
        let unknown = PathBuf::from("LICENSE");
        assert_eq!(made(&path), want(&path, Selection::Crates(both)));
        assert_eq!(made(&unknown), want(&unknown, Selection::Workspace));
    }

    #[test]
    fn a_change_reaches_its_dependents_nothing_reaches_no_crate_and_everything_the_workspace() {
        let graph = PackageGraph::from_metadata(METADATA).unwrap();
        let board = Affected::reached(&graph, &[Reach::Package(BOARD.parse().unwrap()), NOTHING]);
        let reached = BTreeSet::from([BOARD.parse().unwrap(), EVAL.parse().unwrap()]);
        assert_eq!(board, Selection::Crates(reached));
        assert!(Affected::reached(&graph, &[NOTHING]).reaching().is_none());
        assert_eq!(
            Affected::reached(&graph, &[NOTHING, EVERYTHING]),
            Selection::Workspace
        );
    }
}
