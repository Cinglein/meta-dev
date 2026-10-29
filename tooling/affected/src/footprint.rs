use std::collections::BTreeSet;
use std::path::PathBuf;

use task::QueueSettings;
use task::Selection;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Footprint {
    reach: Selection,
    paths: BTreeSet<PathBuf>,
    hot: BTreeSet<PathBuf>,
}

impl Footprint {
    #[must_use]
    pub fn new(reach: Selection, paths: BTreeSet<PathBuf>, queue: &QueueSettings) -> Footprint {
        Footprint {
            hot: queue.hot_among(&paths),
            reach,
            paths,
        }
    }

    #[must_use]
    pub fn meets(&self, other: &Footprint) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use settings::EmbeddedDefaults;
    use task::{Policy, Selection};

    use super::Footprint;

    type Side = (Option<&'static [&'static str]>, &'static [&'static str]);
    const BOARD: &str = "crates/board/src";
    const ROWS: [(Side, Side, bool); 6] = [
        ((None, &["a"]), (Some(&["eval"]), &["b"]), true),
        (
            (Some(&["board", "eval"]), &["crates/board/src/x.rs"]),
            (Some(&["eval"]), &["crates/eval/src/y.rs"]),
            true,
        ),
        (
            (Some(&["board"]), &["goldens/snapshots/board/a"]),
            (Some(&["eval"]), &["goldens/snapshots/eval/b"]),
            true,
        ),
        ((Some(&[]), &["notes/x"]), (Some(&[]), &["notes/x"]), true),
        (
            (Some(&["board"]), &["crates/board/src/lib.rs"]),
            (Some(&["eval"]), &["crates/eval/src/lib.rs"]),
            false,
        ),
        ((Some(&["board"]), &[BOARD]), (Some(&["eval"]), &["e"]), false),
    ];

    fn footprint(side: &Side) -> Footprint {
        let reach = side.0.map_or(Selection::Workspace, |names| {
            Selection::Crates(names.iter().map(|name| name.parse().unwrap()).collect())
        });
        let paths = side.1.iter().map(PathBuf::from).collect();
        Footprint::new(reach, paths, Policy::defaults().queue())
    }

    #[test]
    fn footprints_meet_on_workspace_a_crate_a_hot_path_or_a_path_and_not_otherwise() {
        let meets = ROWS.map(|(a, b, _)| footprint(&a).meets(&footprint(&b)));
        let flipped = ROWS.map(|(a, b, _)| footprint(&b).meets(&footprint(&a)));
        assert_eq!(meets, ROWS.map(|row| row.2));
        assert_eq!(flipped, meets);
    }
}
