use meta_severity::{ActionLevels, PromoteAfter};
use serde::Deserialize;

use crate::policy::policy_failure::PolicyFailure;

#[derive(Debug, Deserialize)]
pub struct FallbackKeys {
    #[serde(default)]
    promote_after: Option<PromoteAfter>,
    #[serde(default)]
    act: Option<ActionLevels>,
}

impl FallbackKeys {
    const PROMOTE_AFTER: &str = "promote_after";
    const ACT: &str = "act";

    pub fn read(text: &str) -> Result<FallbackKeys, PolicyFailure> {
        toml::from_str(text).map_err(PolicyFailure::Fallback)
    }

    pub fn promote_after(self) -> Result<PromoteAfter, PolicyFailure> {
        self.promote_after
            .ok_or(PolicyFailure::Missing(Self::PROMOTE_AFTER))
    }

    pub fn act(self) -> Result<ActionLevels, PolicyFailure> {
        self.act.ok_or(PolicyFailure::Missing(Self::ACT))
    }
}
