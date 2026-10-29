use serde::{Deserialize, Serialize};
use strum::Display;

use super::external_class::ExternalClass;

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
    #[must_use]
    pub const fn may_depend_on(self, target: CrateCategory) -> bool {
        match self {
            CrateCategory::Core | CrateCategory::Port => matches!(target, CrateCategory::Core),
            CrateCategory::Shell | CrateCategory::Bin => matches!(
                target,
                CrateCategory::Core | CrateCategory::Port | CrateCategory::Shell
            ),
            CrateCategory::TestSupport => {
                !matches!(target, CrateCategory::Bin | CrateCategory::Tooling)
            }
            CrateCategory::Tooling => !matches!(target, CrateCategory::Bin),
        }
    }

    #[must_use]
    pub const fn may_use(self, class: &ExternalClass) -> bool {
        match class {
            ExternalClass::General => true,
            ExternalClass::Framework => matches!(self, CrateCategory::Bin | CrateCategory::Tooling),
            ExternalClass::Domain(_) => matches!(self, CrateCategory::Port),
        }
    }
}
