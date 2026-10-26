use strum::{Display, EnumString};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Display, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum TopDivision {
    FeaturesAndMaintenance,
    PerformanceAndCost,
    ArchitectureAndPlanning,
}

#[cfg(test)]
mod tests {
    use enum_map::EnumMap;

    use super::TopDivision;
    use crate::budget::{Interval, MemoryShare, MutantMillis, MutationCadence};
    use crate::policy::{Floor, Limit, Share};

    const DIVISIONS: [TopDivision; 3] = [
        TopDivision::FeaturesAndMaintenance,
        TopDivision::PerformanceAndCost,
        TopDivision::ArchitectureAndPlanning,
    ];
    const ENTRIES_PER_DIVISION: [usize; 3] = [17, 16, 0];

    #[test]
    fn every_policy_entry_sits_in_one_division_and_none_enforced_in_code_is_architecture() {
        let placed: Vec<TopDivision> = EnumMap::from_fn(Limit::division)
            .into_values()
            .chain(EnumMap::from_fn(Floor::division).into_values())
            .chain(EnumMap::from_fn(Share::division).into_values())
            .chain(EnumMap::from_fn(Interval::division).into_values())
            .chain(EnumMap::from_fn(MemoryShare::division).into_values())
            .chain([MutantMillis::DIVISION, MutationCadence::DIVISION])
            .collect();
        let count =
            |division: TopDivision| placed.iter().filter(|placed| **placed == division).count();
        assert_eq!(DIVISIONS.map(count), ENTRIES_PER_DIVISION);
    }
}
