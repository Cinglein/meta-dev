use enum_map::Enum;
use serde::{Deserialize, Serialize};

use meta_schema::TopDivision;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum Limit {
    Parameters,
    Nesting,
    Reexports,
    PrivateFnsPerType,
    EdgesPerVertex,
    TestsPerFile,
    AssertionsPerTest,
    LinesPerTest,
    LiteralsPerTest,
    LargestInteger,
    SlowestShown,
    SccItems,
    TypesPerMethodShape,
}

impl Limit {
    #[must_use]
    pub const fn division(self) -> TopDivision {
        match self {
            Limit::SlowestShown => TopDivision::PerformanceAndCost,
            Limit::Reexports
            | Limit::EdgesPerVertex
            | Limit::SccItems
            | Limit::Parameters
            | Limit::Nesting
            | Limit::PrivateFnsPerType
            | Limit::TestsPerFile
            | Limit::AssertionsPerTest
            | Limit::LinesPerTest
            | Limit::LiteralsPerTest
            | Limit::LargestInteger
            | Limit::TypesPerMethodShape => TopDivision::FeaturesAndMaintenance,
        }
    }
}

#[cfg(test)]
mod tests {
    use enum_map::EnumMap;
    use meta_schema::TopDivision;

    use super::Limit;
    use crate::budget::{Interval, MemoryShare, MutantMillis, MutationCadence};
    use crate::policy::{Floor, Share};

    const DIVISIONS: [TopDivision; 3] = [
        TopDivision::FeaturesAndMaintenance,
        TopDivision::PerformanceAndCost,
        TopDivision::ArchitectureAndPlanning,
    ];
    const ENTRIES_PER_DIVISION: [usize; 3] = [18, 20, 0];

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
