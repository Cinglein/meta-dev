mod changes;
mod crate_load;
mod packing;
mod reach;
mod tested_crates;

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
use task::Workspace;

#[derive(Debug)]
pub struct Affected;

impl Affected {
    pub fn selection(
        workspace: &Workspace,
        changes: Changes,
        base: &BranchName,
    ) -> Result<Selection, Failure> {
        Ok(Self::reached(
            workspace.packages(),
            &changes.reaches(workspace, base)?,
        ))
    }

    #[must_use]
    pub fn footprint(
        packages: &PackageGraph,
        reaches: &[Reach],
        paths: Vec<PathBuf>,
        policy: &Policy,
    ) -> Footprint {
        Footprint::new(
            Self::reached(packages, reaches),
            paths.into_iter().collect(),
            policy,
        )
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
        let made =
            |reach: Reach| Affected::footprint(&graph, &[reach], vec![path.clone()], &policy);
        let want =
            |reach: Selection| Footprint::new(reach, BTreeSet::from([path.clone()]), &policy);
        let both = BTreeSet::from([BOARD.parse().unwrap(), EVAL.parse().unwrap()]);
        assert_eq!(
            made(Reach::Package(BOARD.parse().unwrap())),
            want(Selection::Crates(both))
        );
        assert_eq!(made(EVERYTHING), want(Selection::Workspace));
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
