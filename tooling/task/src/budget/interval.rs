use enum_map::Enum;
use serde::{Deserialize, Serialize};
use strum::IntoStaticStr;

use meta_schema::TopDivision;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, IntoStaticStr, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
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
    ReferenceCpu,
}

impl Interval {
    #[must_use]
    pub const fn division(self) -> TopDivision {
        TopDivision::PerformanceAndCost
    }
}
