use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::micros::Micros;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TraceId(u64);

impl TraceId {
    pub const SHOWN_LEN: usize = 16;
    const HEX: u32 = 16;

    #[must_use]
    pub fn new() -> Self {
        Self(Micros::now().micros())
    }

    #[must_use]
    pub fn file_in(self, dir: &Path, extension: &str) -> PathBuf {
        dir.join(format!("{self}-{}", std::process::id())).with_extension(extension)
    }
}

impl Default for TraceId {
    fn default() -> Self {
        Self::new()
    }
}

impl FromStr for TraceId {
    type Err = std::num::ParseIntError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        u64::from_str_radix(text, Self::HEX).map(Self)
    }
}

impl fmt::Display for TraceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:016x}", self.0)
    }
}
