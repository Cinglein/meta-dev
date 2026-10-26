use serde::Deserialize;

use crate::request::local::Sweep;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GcMode {
    DryRun,
    Free,
}

impl GcMode {
    pub fn sweep(self) -> Sweep {
        match self {
            Self::DryRun => Sweep::Planned,
            Self::Free => Sweep::Done,
        }
    }
}
