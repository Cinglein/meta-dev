use serde::Deserialize;

use crate::request::local::Sweep;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(from = "String")]
pub enum GcMode {
    DryRun,
    Free,
    Disabled,
}

impl GcMode {
    const DRY_RUN: &'static str = "dry-run";
    const FREE: &'static str = "free";

    pub fn sweep(self) -> Option<Sweep> {
        match self {
            Self::DryRun => Some(Sweep::Planned),
            Self::Free => Some(Sweep::Done),
            Self::Disabled => None,
        }
    }
}

impl From<String> for GcMode {
    fn from(switch: String) -> Self {
        match switch.as_str() {
            Self::DRY_RUN => Self::DryRun,
            Self::FREE => Self::Free,
            _ => Self::Disabled,
        }
    }
}
