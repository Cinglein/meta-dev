use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::full_run::FullRun;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MutationCadence {
    full_run: BTreeSet<FullRun>,
}

impl MutationCadence {
    #[must_use]
    pub fn due(&self, trigger: Option<FullRun>) -> Option<FullRun> {
        trigger.filter(|run| self.full_run.contains(run))
    }
}

#[cfg(test)]
mod tests {
    use super::{FullRun, MutationCadence};
    const WEEKLY: &str = "full_run = [\"weekly\", \"on_demand\"]\n";
    const UNKNOWN: &str = "full_run = [\"hourly\"]\n";
    const TRIGGERS: [Option<FullRun>; 4] = [
        Some(FullRun::EveryMainPush),
        Some(FullRun::Weekly),
        Some(FullRun::OnDemand),
        None,
    ];
    const DUE: [Option<FullRun>; 4] = [None, Some(FullRun::Weekly), Some(FullRun::OnDemand), None];

    #[test]
    fn a_full_run_is_due_only_for_a_trigger_the_policy_lists() {
        let cadence: MutationCadence = toml::from_str(WEEKLY).unwrap();
        assert_eq!(TRIGGERS.map(|trigger| cadence.due(trigger)), DUE);
        assert!(toml::from_str::<MutationCadence>(UNKNOWN).is_err());
    }
}
