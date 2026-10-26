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
        _job: Interval,
    ) -> CrateLoad {
        let each = policy.mutant_millis().for_crate(&name);
        CrateLoad::new(name, mutants, each, policy)
    }

    pub fn listed(listing: CommandLine, policy: &Policy) -> Result<Vec<CrateLoad>, Failure> {
        let json = listing
            .argument(MutantsArgument::List)
            .argument(MutantsArgument::Json)
            .captured::<Stdout>()?;
        Ok(ListedMutant::tally(&json)?
            .into_iter()
            .map(|(name, mutants)| {
                let each = policy.mutant_millis().for_crate(&name);
                CrateLoad::new(name, mutants, each, policy)
            })
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
