use enum_map::Enum;
use serde::{Deserialize, Serialize};

use meta_schema::TopDivision;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum Floor {
    FnName,
    LifetimeName,
    CutSide,
    PriceSample,
    DiffFactor,
    PriceTolerance,
}

impl Floor {
    #[must_use]
    pub const fn division(self) -> TopDivision {
        match self {
            Floor::FnName | Floor::LifetimeName | Floor::CutSide => {
                TopDivision::FeaturesAndMaintenance
            }
            Floor::PriceSample | Floor::DiffFactor | Floor::PriceTolerance => {
                TopDivision::PerformanceAndCost
            }
        }
    }
}
