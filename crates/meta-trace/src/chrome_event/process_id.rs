use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProcessId(u32);

impl ProcessId {
    pub fn own() -> Self {
        Self(std::process::id())
    }
}
