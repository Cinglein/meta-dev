mod forming;

use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::policy::Policy;
use crate::selection::Selection;
use forming::Forming;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Footprint {
    reach: Selection,
    paths: BTreeSet<PathBuf>,
    hot: BTreeSet<PathBuf>,
}

impl Footprint {
    #[must_use]
    pub fn new(reach: Selection, paths: BTreeSet<PathBuf>, policy: &Policy) -> Footprint {
        Footprint {
            hot: policy.queue().hot_among(&paths),
            reach,
            paths,
        }
    }

    #[must_use]
    pub fn meets(&self, other: &Footprint) -> bool {
        self.reach.common(&other.reach).is_some()
            || !self.hot.is_disjoint(&other.hot)
            || !self.paths.is_disjoint(&other.paths)
    }

    #[must_use]
    pub fn batches<'queue>(
        queue: &'queue [Footprint],
        policy: &Policy,
    ) -> Vec<Vec<&'queue Footprint>> {
        Forming::batches(queue.iter().collect(), policy, Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use settings::EmbeddedDefaults;

    use super::Footprint;
    use crate::policy::Policy;
    use crate::selection::Selection;

    type Side = (Option<&'static [&'static str]>, &'static [&'static str]);
    const CRATES: [&str; 3] = ["a", "b", "c"];
    const LONGEST_QUEUE: usize = 5;
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
        ((Some(&["board"]), &["b"]), (Some(&["eval"]), &["e"]), false),
    ];

    impl Footprint {
        fn on(side: &Side) -> Footprint {
            let reach = side.0.map_or(Selection::Workspace, |names| {
                Selection::Crates(names.iter().map(|name| name.parse().unwrap()).collect())
            });
            let paths = side.1.iter().map(PathBuf::from).collect();
            Footprint::new(reach, paths, &Policy::defaults())
        }

        fn alphabet() -> Vec<Footprint> {
            let named = CRATES.map(|name| {
                let paths = BTreeSet::from([PathBuf::from(name)]);
                let reach = Selection::Crates(BTreeSet::from([name.parse().unwrap()]));
                Footprint::new(reach, paths, &Policy::defaults())
            });
            let whole = Footprint::new(Selection::Workspace, BTreeSet::new(), &Policy::defaults());
            named.into_iter().chain([whole]).collect()
        }

        fn queues(alphabet: &[Footprint], length: usize) -> Vec<Vec<Footprint>> {
            (0..length).fold(vec![Vec::new()], |queues, _| {
                queues
                    .iter()
                    .flat_map(|queue| {
                        alphabet
                            .iter()
                            .map(|next| [queue.clone(), vec![next.clone()]].concat())
                    })
                    .collect()
            })
        }
    }

    #[test]
    fn footprints_meet_on_workspace_a_crate_a_hot_path_or_a_path_and_not_otherwise() {
        let meets = ROWS.map(|(a, b, _)| Footprint::on(&a).meets(&Footprint::on(&b)));
        let flipped = ROWS.map(|(a, b, _)| Footprint::on(&b).meets(&Footprint::on(&a)));
        assert_eq!(meets, ROWS.map(|row| row.2));
        assert_eq!(flipped, meets);
    }

    #[test]
    fn every_formed_batch_is_pairwise_disjoint_within_the_size_and_keeps_every_pull() {
        let policy = Policy::defaults();
        let alphabet = Footprint::alphabet();
        let whole = &alphabet[CRATES.len()];
        let sound = (0..=LONGEST_QUEUE)
            .flat_map(|length| Footprint::queues(&alphabet, length))
            .all(|queue| {
                let batches = Footprint::batches(&queue, &policy);
                batches.iter().all(|batch| {
                    policy
                        .queue()
                        .admits(&batch[..batch.len().saturating_sub(1)])
                        && batch.iter().enumerate().all(|(at, member)| {
                            batch[at + 1..]
                                .iter()
                                .all(|other| member != other && *member != whole && *other != whole)
                        })
                }) && batches.concat().len() == queue.len()
            });
        assert!(sound);
    }

    #[test]
    fn a_pull_joins_the_first_batch_it_is_disjoint_from_and_a_workspace_pull_stands_alone() {
        let alphabet = Footprint::alphabet();
        let [a, b, whole] = [0, 1, CRATES.len()].map(|at| alphabet[at].clone());
        let queue = [a.clone(), whole.clone(), b.clone()];
        let expected = vec![vec![&a, &b], vec![&whole]];
        assert_eq!(Footprint::batches(&queue, &Policy::defaults()), expected);
    }
}
