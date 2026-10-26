mod listed_mutant;

use listed_mutant::ListedMutant;
use task::CrateName;
use task::Failure;
use task::Millis;
use task::MutantCount;
use task::{CommandLine, MutantsArgument, Stdout};
use task::{Interval, Policy};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrateLoad {
    name: CrateName,
    cost: Millis,
}

impl CrateLoad {
    #[must_use]
    pub fn new(
        name: CrateName,
        mutants: MutantCount,
        millis_each: Millis,
        policy: &Policy,
    ) -> CrateLoad {
        let cost = policy
            .interval(Interval::PerCrate)
            .plus(millis_each.times(mutants.into()));
        CrateLoad { name, cost }
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
        self.cost
    }
}
