mod bin;

use std::cmp::Reverse;

use bin::Bin;

use crate::crate_load::CrateLoad;
use task::Millis;
use task::{Interval, Policy, Share};
use task::{Shard, Slice, SliceCount};

#[derive(Debug)]
pub struct Packing;

impl Packing {
    #[must_use]
    pub fn target(policy: &Policy, job: Interval) -> Millis {
        policy
            .share(Share::ShardHeadroom)
            .portion_of(policy.interval(job))
            .checked_minus(policy.interval(Interval::JobSetup))
            .unwrap_or_default()
    }

    pub fn shards(loads: &[CrateLoad], policy: &Policy, job: Interval) -> Vec<Shard> {
        let target = Self::target(policy, job);
        let slices = loads.iter().flat_map(|load| {
            load.slices(target)
                .into_iter()
                .flat_map(move |count| Self::slices(load, count))
        });
        let mut whole: Vec<&CrateLoad> = loads
            .iter()
            .filter(|load| load.slices(target).is_none())
            .collect();
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
    use std::time::Duration;
    use task::{CrateName, Interval, Millis, MutantCount, Policy, Share};
    const PR_SEVEN_CRATE: &str = "meta-cli";
    const PR_SEVEN_MUTANTS: &str = "67";
    const PR_SEVEN_SECONDS_PER_MUTANT_ON_CI: f64 = 5.8;
    const LOADS: [(&str, usize, Interval); 4] = [
        ("a", 2, Interval::FullJob),
        ("b", 1, Interval::DiffJob),
        ("c", 1, Interval::SampleEvery),
        ("d", 2, Interval::TestTimeout),
    ];
    const SHARDS: [&str; 4] = ["a/0/2", "a/1/2", "b", "c+d"];
    const OVERHEAD_PRICED: (usize, usize) = (27, 7);

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
        let shards: Vec<String> = Packing::shards(&loads, &policy, Interval::FullJob)
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(shards, SHARDS);
    }

    #[test]
    fn every_slice_pays_the_crate_overhead_and_still_fits_the_target() {
        let policy = Policy::defaults();
        let (mutants, slices) = OVERHEAD_PRICED;
        let each = policy.interval(Interval::PerCrate);
        let load = CrateLoad::new(
            PR_SEVEN_CRATE.parse().unwrap(),
            MutantCount::from(mutants),
            each,
            &policy,
        );
        let shards = Packing::shards(&[load], &policy, Interval::DiffJob);
        assert_eq!(shards.len(), slices);
        assert!(
            each.plus(each.times(mutants.div_ceil(slices)))
                <= Packing::target(&policy, Interval::DiffJob)
        );
    }

    #[test]
    fn a_meta_cli_diff_the_size_of_pr_7_fits_the_headroom_of_its_job_at_the_cost_measured_on_ci() {
        let policy = Policy::defaults();
        let measured = Duration::from_secs_f64(PR_SEVEN_SECONDS_PER_MUTANT_ON_CI).as_millis();
        let measured = Millis::new(u64::try_from(measured).unwrap());
        let mutants: usize = PR_SEVEN_MUTANTS.parse().unwrap();
        let name: CrateName = PR_SEVEN_CRATE.parse().unwrap();
        let job = Interval::DiffJob;
        let load = CrateLoad::priced(name, MutantCount::from(mutants), &policy, job);
        let shards = Packing::shards(&[load], &policy, job).len();
        let spent = policy
            .interval(Interval::JobSetup)
            .plus(policy.interval(Interval::PerCrate))
            .plus(measured.times(mutants.div_ceil(shards)));
        let headroom = policy.share(Share::ShardHeadroom);
        assert!(spent <= headroom.portion_of(policy.interval(job)));
    }
}
