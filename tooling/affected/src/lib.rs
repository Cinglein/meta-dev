mod changes;
mod reach;

pub use changes::Changes;

use reach::Reach;
use task::Failure;
use task::PackageGraph;
use task::Selection;
use task::Workspace;

#[derive(Debug)]
pub struct Affected;

impl Affected {
    pub fn selection(workspace: &Workspace, changes: Changes) -> Result<Selection, Failure> {
        Ok(Self::reached(
            workspace.packages(),
            &changes.reaches(workspace)?,
        ))
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
    use super::{Affected, Reach, Selection};
    use std::collections::BTreeSet;
    use task::PackageGraph;
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
    fn a_change_reaches_its_dependents_nothing_reaches_no_crate_and_everything_the_workspace() {
        let graph = PackageGraph::from_metadata(METADATA).unwrap();
        let board = Affected::reached(&graph, &[Reach::Package(BOARD.parse().unwrap()), NOTHING]);
        let reached = BTreeSet::from([BOARD.parse().unwrap(), EVAL.parse().unwrap()]);
        assert_eq!(board, Selection::Crates(reached));
        assert!(Affected::reached(&graph, &[NOTHING]).is_empty());
        assert_eq!(
            Affected::reached(&graph, &[NOTHING, EVERYTHING]),
            Selection::Workspace
        );
    }
}
