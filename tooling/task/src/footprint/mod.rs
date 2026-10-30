use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::policy::{Policy, Surface};
use crate::selection::Selection;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Footprint {
    reach: Selection,
    paths: BTreeSet<PathBuf>,
    hot: BTreeSet<Surface>,
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
        [&self.reach, &other.reach].contains(&&Selection::Workspace)
            || self.reach.clone().meet(&other.reach).reaching().is_some()
            || !self.hot.is_disjoint(&other.hot)
            || !self.paths.is_disjoint(&other.paths)
    }

    fn fits(&self, batch: &[&Footprint], policy: &Policy) -> bool {
        policy.queue().admits(batch) && batch.iter().all(|member| !member.meets(self))
    }

    #[must_use]
    pub fn batches<'queue>(
        queue: &'queue [Footprint],
        policy: &Policy,
    ) -> Vec<Vec<&'queue Footprint>> {
        queue.iter().fold(Vec::new(), |mut batches, footprint| {
            let at = batches
                .iter()
                .position(|batch| footprint.fits(batch, policy));
            let joined = at
                .and_then(|at| batches.get_mut(at))
                .map(|batch| batch.push(footprint));
            batches.extend(joined.is_none().then_some(vec![footprint]));
            batches
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use settings::EmbeddedDefaults;

    use super::Footprint;
    use crate::policy::Policy;
    use crate::selection::Selection;

    type Side = (Option<&'static [&'static str]>, &'static [&'static str]);
    const WORKSPACE: usize = 7;
    const MEMBERS: usize = 4;
    const LONGEST_QUEUE: usize = 3;
    const ROWS: [(Side, Side, bool); 6] = [
        ((None, &["a"]), (Some(&["eval"]), &["b"]), true),
        (
            (Some(&["board", "eval"]), &["x"]),
            (Some(&["eval"]), &["y"]),
            true,
        ),
        (
            (Some(&["a"]), &["goldens/snapshots/a"]),
            (Some(&["b"]), &["goldens/snapshots/b"]),
            true,
        ),
        ((Some(&[]), &["notes/x"]), (Some(&[]), &["notes/x"]), true),
        (
            (Some(&["a"]), &["crates/a/lib.rs"]),
            (Some(&["b"]), &["crates/b/lib.rs"]),
            false,
        ),
        ((Some(&["a"]), &["x"]), (Some(&["b"]), &["y"]), false),
    ];
    const ALPHABET: [Side; 9] = [
        (Some(&["a"]), &["a"]),
        (Some(&["b"]), &["b"]),
        (Some(&["a", "b"]), &["ab"]),
        (Some(&["c"]), &["goldens/snapshots/c"]),
        (Some(&["d"]), &["goldens/snapshots/d"]),
        (Some(&["e"]), &["shared"]),
        (Some(&["f"]), &["shared"]),
        (None, &[]),
        (Some(&["g"]), &["g"]),
    ];
    const GROUPS: [(&[usize], &[&[usize]]); 3] = [
        (&[0, WORKSPACE, 1], &[&[0, 1], &[WORKSPACE]]),
        (&[0, 2, 1], &[&[0, 1], &[2]]),
        (&[0, 1, 3, 5, 8], &[&[0, 1, 3, 5], &[8]]),
    ];

    impl Footprint {
        fn on(side: &Side) -> Footprint {
            let reach = side.0.map_or(Selection::Workspace, |names| {
                Selection::Crates(names.iter().map(|name| name.parse().unwrap()).collect())
            });
            Footprint::new(
                reach,
                side.1.iter().map(PathBuf::from).collect(),
                &Policy::defaults(),
            )
        }

        fn queues(alphabet: &[Footprint], length: usize) -> Vec<Vec<Footprint>> {
            (0..length).fold(vec![Vec::new()], |queues, _| {
                let next = |queue: &Vec<Footprint>| {
                    alphabet
                        .iter()
                        .map(|one| [queue.clone(), vec![one.clone()]].concat())
                        .collect::<Vec<_>>()
                };
                queues.iter().flat_map(next).collect()
            })
        }
    }

    #[test]
    fn footprints_meet_on_workspace_a_crate_a_hot_tree_or_a_path_and_not_otherwise() {
        let meets = ROWS.map(|(a, b, _)| Footprint::on(&a).meets(&Footprint::on(&b)));
        let flipped = ROWS.map(|(a, b, _)| Footprint::on(&b).meets(&Footprint::on(&a)));
        assert_eq!((meets, flipped), (ROWS.map(|row| row.2), meets));
    }

    #[test]
    fn every_formed_batch_is_pairwise_disjoint_within_the_size_and_keeps_every_pull() {
        let alphabet = ALPHABET.map(|side| Footprint::on(&side));
        let policy = Policy::defaults();
        let sound = (0..=LONGEST_QUEUE)
            .flat_map(|length| Footprint::queues(&alphabet, length))
            .all(|queue| {
                let batches = Footprint::batches(&queue, &policy);
                let apart = |batch: &Vec<&Footprint>| {
                    batch.len() <= MEMBERS
                        && batch.iter().enumerate().all(|(at, member)| {
                            batch[at + 1..].iter().all(|other| !member.meets(other))
                        })
                };
                batches.iter().all(apart) && batches.concat().len() == queue.len()
            });
        assert!(sound);
    }

    #[test]
    fn a_later_disjoint_pull_joins_an_earlier_batch_the_waiting_head_starts_the_next_and_a_batch_is_full_at_its_size()
     {
        let alphabet = ALPHABET.map(|side| Footprint::on(&side));
        let grouped = GROUPS.map(|(queue, _)| {
            let pulls: Vec<Footprint> = queue.iter().map(|at| alphabet[*at].clone()).collect();
            let place =
                |member: &&Footprint| alphabet.iter().position(|one| one == *member).unwrap();
            Footprint::batches(&pulls, &Policy::defaults())
                .iter()
                .map(|batch| batch.iter().map(place).collect())
                .collect::<Vec<Vec<usize>>>()
        });
        assert_eq!(
            grouped,
            GROUPS.map(|(_, expected)| expected
                .iter()
                .map(|batch| batch.to_vec())
                .collect::<Vec<_>>())
        );
    }
}
