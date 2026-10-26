mod bin;

use std::cmp::Reverse;
use std::num::NonZeroUsize;

use bin::Bin;

use crate::crate_load::CrateLoad;
use task::Millis;
use task::{Shard, Slice, SliceCount};

#[derive(Debug)]
pub struct Packing;

impl Packing {
    pub fn shards(loads: &[CrateLoad], target: Millis) -> Vec<Shard> {
        let slice_count = |load: &CrateLoad| load.cost().div_ceil(target);
        let slices = loads
            .iter()
            .filter(|load| slice_count(load) > 1)
            .flat_map(|load| {
                NonZeroUsize::new(slice_count(load))
                    .map(SliceCount::from)
                    .into_iter()
                    .flat_map(move |count| Self::slices(load, count))
            });
        let mut whole: Vec<&CrateLoad> =
            loads.iter().filter(|load| slice_count(load) <= 1).collect();
        whole.sort_by_key(|load| Reverse(load.cost()));
        let bins = whole.into_iter().fold(Vec::<Bin>::new(), |mut bins, load| {
            if let Some(bin) = bins.last_mut().filter(|bin| bin.fits(load.cost(), target)) {
                bin.take(load.name().clone(), load.cost());
                bins
            } else {
                bins.push(Bin::holding(load.name().clone(), load.cost()));
                bins
            }
        });
        slices.chain(bins.into_iter().map(Bin::shard)).collect()
    }

    fn slices(load: &CrateLoad, count: SliceCount) -> impl Iterator<Item = Shard> {
        let name = load.name().clone();
        Slice::all(count).map(move |slice| Shard::Slice {
            crate_name: name.clone(),
            slice,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{CrateLoad, Packing};
    use settings::EmbeddedDefaults;
    use task::{Interval, MutantCount, Policy};
    const LOADS: [(&str, usize, Interval); 4] = [
        ("a", 1, Interval::PerShard),
        ("b", 1, Interval::PerSlice),
        ("c", 1, Interval::SampleEvery),
        ("d", 2, Interval::TestTimeout),
    ];
    const SHARDS: [&str; 4] = ["a/0/2", "a/1/2", "b", "c+d"];

    #[test]
    fn crates_costlier_than_a_shard_are_sliced_and_the_rest_are_packed_largest_first() {
        let policy = Policy::defaults();
        let loads = LOADS.map(|(name, mutants, each)| {
            CrateLoad::new(
                name.parse().unwrap(),
                MutantCount::from(mutants),
                policy.interval(each),
                &policy,
            )
        });
        let shards: Vec<String> = Packing::shards(&loads, policy.interval(Interval::PerShard))
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(shards, SHARDS);
    }
}
