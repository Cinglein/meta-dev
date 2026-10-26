use enum_map::Enum;
use serde::{Deserialize, Serialize};

use crate::top_division::TopDivision;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum Interval {
    PerTest,
    Total,
    SampleEvery,
    DiscoverEvery,
    PerCrate,
    TestTimeout,
    FullJob,
    DiffJob,
    JobSetup,
}

impl Interval {
    #[must_use]
    pub const fn division(self) -> TopDivision {
        TopDivision::PerformanceAndCost
    }
}
