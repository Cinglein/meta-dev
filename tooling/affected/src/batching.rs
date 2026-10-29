use task::BatchSize;

use crate::footprint::Footprint;

#[derive(Debug)]
pub struct Batching;

impl Batching {
    #[must_use]
    pub fn group(queue: &[Footprint], size: BatchSize) -> Vec<Vec<&Footprint>> {
        Self::grouped(queue.iter().collect(), size, Vec::new())
    }

    fn grouped<'queue>(
        waiting: Vec<&'queue Footprint>,
        size: BatchSize,
        formed: Vec<Vec<&'queue Footprint>>,
    ) -> Vec<Vec<&'queue Footprint>> {
        if waiting.is_empty() {
            return formed;
        }
        let (batch, rest) = waiting.into_iter().fold(
            (Vec::new(), Vec::new()),
            |(mut batch, mut rest): (Vec<&Footprint>, Vec<&Footprint>), footprint| {
                if size.admits(&batch) && batch.iter().all(|member| !member.meets(footprint)) {
                    batch.push(footprint);
                } else {
                    rest.push(footprint);
                }
                (batch, rest)
            },
        );
        Self::grouped(rest, size, [formed, vec![batch]].concat())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use settings::EmbeddedDefaults;
    use task::{Policy, Selection};

    use super::{Batching, Footprint};

    const CRATES: [&str; 3] = ["a", "b", "c"];

    fn alphabet() -> Vec<Footprint> {
        let queue = Policy::defaults().queue().clone();
        let on = |name: &str| {
            let paths = BTreeSet::from([PathBuf::from(name)]);
            let reach = Selection::Crates(BTreeSet::from([name.parse().unwrap()]));
            Footprint::new(reach, paths, &queue)
        };
        let whole = Footprint::new(Selection::Workspace, BTreeSet::new(), &queue);
        CRATES.map(on).into_iter().chain([whole]).collect()
    }

    fn queues(alphabet: &[Footprint], length: usize) -> Vec<Vec<Footprint>> {
        (0..length).fold(vec![Vec::new()], |queues, _| {
            queues
                .iter()
                .flat_map(|queue| alphabet.iter().map(|next| [queue.clone(), vec![next.clone()]].concat()))
                .collect()
        })
    }

    #[test]
    fn every_formed_batch_is_pairwise_disjoint_within_the_size_and_keeps_every_pull() {
        let size = Policy::defaults().queue().members();
        let alphabet = alphabet();
        let whole = &alphabet[CRATES.len()];
        let sound = (0..=5)
            .flat_map(|length| queues(&alphabet, length))
            .all(|queue| {
                let batches = Batching::group(&queue, size);
                batches.iter().all(|batch| {
                    batch.len() <= 4
                        && batch.iter().enumerate().all(|(at, member)| {
                            batch[at + 1..].iter().all(|other| {
                                member != other && *member != whole && *other != whole
                            })
                        })
                }) && batches.concat().len() == queue.len()
            });
        assert!(sound);
    }

    #[test]
    fn a_pull_joins_the_first_batch_it_is_disjoint_from_and_a_workspace_pull_stands_alone() {
        let size = Policy::defaults().queue().members();
        let alphabet = alphabet();
        let [a, b, _, whole] = [0, 1, 2, 3].map(|at| alphabet[at].clone());
        let queue = [a.clone(), whole.clone(), b.clone()];
        let expected = vec![vec![&a, &b], vec![&whole]];
        assert_eq!(Batching::group(&queue, size), expected);
    }
}
