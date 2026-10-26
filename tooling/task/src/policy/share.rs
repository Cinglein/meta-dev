use enum_map::Enum;
use serde::{Deserialize, Serialize};

use crate::top_division::TopDivision;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum Share {
    TestsPerHundredFiles,
    TestLinePercent,
    CutConductance,
}

impl Share {
    #[must_use]
    pub const fn division(self) -> TopDivision {
        match self {
            Share::TestsPerHundredFiles | Share::TestLinePercent => {
                TopDivision::FeaturesAndMaintenance
            }
            Share::CutConductance => TopDivision::ArchitectureAndPlanning,
        }
    }
}
