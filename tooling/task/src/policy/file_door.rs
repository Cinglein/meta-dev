use serde::{Deserialize, Serialize};

use crate::crate_name::CrateName;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileDoor {
    crates: Vec<CrateName>,
    #[serde(default)]
    dev_only: Vec<CrateName>,
}

impl FileDoor {
    #[must_use]
    pub fn covers(&self, name: &CrateName) -> bool {
        self.crates.contains(name)
    }

    #[must_use]
    pub fn dev_only(&self) -> &[CrateName] {
        &self.dev_only
    }
}
