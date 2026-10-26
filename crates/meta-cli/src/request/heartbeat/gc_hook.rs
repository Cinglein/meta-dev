use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GcHook {
    NotBuilt,
}

impl GcHook {
    pub fn run() -> Self {
        Self::NotBuilt
    }
}
