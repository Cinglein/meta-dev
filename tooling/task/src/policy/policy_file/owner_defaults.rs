use meta_severity::{ActionLevels, PromoteAfter};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct OwnerDefaults {
    promote_after: PromoteAfter,
    act: ActionLevels,
}

impl OwnerDefaults {
    const POLICY: &str = include_str!("../../../../policy.toml");

    pub fn embedded() -> Option<OwnerDefaults> {
        toml::from_str(Self::POLICY).ok()
    }

    pub fn promote_after(&self) -> PromoteAfter {
        self.promote_after
    }

    pub fn act(self) -> ActionLevels {
        self.act
    }
}
