use enum_map::Enum;
use serde::{Deserialize, Serialize};

use crate::top_division::TopDivision;

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
            | Limit::LargestInteger => TopDivision::FeaturesAndMaintenance,
        }
    }
}
