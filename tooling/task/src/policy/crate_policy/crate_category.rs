use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Display, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum CrateCategory {
    Core,
    Port,
    Shell,
    Bin,
    Tooling,
    TestSupport,
}

impl CrateCategory {
    const PURE: &[CrateCategory] = &[CrateCategory::Core];
    const WIRED: &[CrateCategory] = &[
        CrateCategory::Core,
        CrateCategory::Port,
        CrateCategory::Shell,
    ];
    const TESTED: &[CrateCategory] = &[
        CrateCategory::Core,
        CrateCategory::Port,
        CrateCategory::Shell,
        CrateCategory::TestSupport,
    ];
    const BUILT: &[CrateCategory] = &[
        CrateCategory::Core,
        CrateCategory::Port,
        CrateCategory::Shell,
        CrateCategory::Tooling,
        CrateCategory::TestSupport,
    ];

    #[must_use]
    pub const fn reaches(self) -> &'static [CrateCategory] {
        match self {
            CrateCategory::Core | CrateCategory::Port => Self::PURE,
            CrateCategory::Shell | CrateCategory::Bin => Self::WIRED,
            CrateCategory::TestSupport => Self::TESTED,
            CrateCategory::Tooling => Self::BUILT,
        }
    }
}
