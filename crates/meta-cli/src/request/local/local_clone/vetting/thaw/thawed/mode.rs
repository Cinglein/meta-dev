use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Mode(u32);

impl Mode {
    const OWNER_ALL: u32 = 0o700;

    pub fn owner_writable(self) -> Self {
        Self(self.0 | Self::OWNER_ALL)
    }

    pub fn new(bits: u32) -> Self {
        Self(bits)
    }

    pub fn bits(self) -> u32 {
        self.0
    }
}
