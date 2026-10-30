mod listed_mutant;

use std::num::NonZeroUsize;

use listed_mutant::ListedMutant;
use task::CrateName;
use task::Failure;
use task::Millis;
use task::MutantCount;
use task::SliceCount;
use task::{CommandLine, MutantsArgument, Stdout};
use task::{Interval, Policy};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrateLoad {
    name: CrateName,
    mutants: MutantCount,
    each: Millis,
    overhead: Millis,
}

impl CrateLoad {
    #[must_use]
    pub fn new(name: CrateName, mutants: MutantCount, each: Millis, policy: &Policy) -> CrateLoad {
        let overhead = policy.interval(Interval::PerCrate);
        CrateLoad {
            name,
            mutants,
            each,
            overhead,
        }
    }

    #[must_use]
    pub fn priced(
        name: CrateName,
        mutants: MutantCount,
        policy: &Policy,
        job: Interval,
    ) -> CrateLoad {
        let each = policy.mutant_price(&name, job);
        policy
            .mutant_millis()
            .unpriced_note(&name)
            .iter()
            .for_each(|note| eprintln!("{note}"));
        CrateLoad::new(name, mutants, each, policy)
    }

    pub fn listed(
        listing: CommandLine,
        policy: &Policy,
        job: Interval,
    ) -> Result<Vec<CrateLoad>, Failure> {
        let json = listing
            .argument(MutantsArgument::List)
            .argument(MutantsArgument::Json)
            .captured::<Stdout>()?;
        Ok(ListedMutant::tally(&json)?
            .into_iter()
            .map(|(name, mutants)| CrateLoad::priced(name, mutants, policy, job))
            .collect())
    }

    #[must_use]
    pub fn name(&self) -> &CrateName {
        &self.name
    }

    #[must_use]
    pub fn cost(&self) -> Millis {
        self.overhead.plus(self.each.times(self.mutants.into()))
    }

    #[must_use]
    pub fn slices(&self, target: Millis) -> Option<SliceCount> {
        let room = target.checked_minus(self.overhead).unwrap_or_default();
        let per_slice = room.times_fitting(self.each).max(1);
        NonZeroUsize::new(usize::from(self.mutants).div_ceil(per_slice))
            .filter(|count| count.get() > 1)
            .map(SliceCount::from)
    }
}

#[cfg(test)]
mod tests {
    use super::CrateLoad;
    use crate::packing::Packing;
    use settings::EmbeddedDefaults;
    use task::{Interval, MutantCount, Policy};
    const UNPRICED: &str = "no-entry-in-the-table";
    const COSTLIEST: &str = "meta-severity";
    const MUTANTS: usize = 30;

    #[test]
    fn a_crate_with_no_price_is_not_split_into_more_shards_than_a_priced_crate_of_the_same_size() {
        let policy = Policy::defaults();
        let shards = |name: &str| {
            let load = CrateLoad::priced(
                name.parse().unwrap(),
                MutantCount::from(MUTANTS),
                &policy,
                Interval::DiffJob,
            );
            Packing::shards(&[load], &policy, Interval::DiffJob).len()
        };
        assert!(shards(UNPRICED) < shards(COSTLIEST));
    }
}
