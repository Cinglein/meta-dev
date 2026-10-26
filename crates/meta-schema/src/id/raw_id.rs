use std::fmt;
use std::hash::BuildHasher;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};

use super::id_failure::IdFailure;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RawId(u64);

impl RawId {
    const HEX_LEN: usize = 16;
    const RADIX: u32 = 16;

    pub fn generate() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(std::collections::hash_map::RandomState::new().hash_one(COUNTER.fetch_add(1, Ordering::Relaxed)))
    }
}

impl FromStr for RawId {
    type Err = IdFailure;

    fn from_str(hex: &str) -> Result<Self, Self::Err> {
        Some(hex)
            .filter(|hex| {
                hex.len() == Self::HEX_LEN && hex.bytes().all(|b| b.is_ascii_digit() || b.is_ascii_lowercase())
            })
            .and_then(|hex| u64::from_str_radix(hex, Self::RADIX).ok())
            .map(Self)
            .ok_or(IdFailure::Malformed)
    }
}

impl fmt::Display for RawId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:0width$x}", self.0, width = Self::HEX_LEN)
    }
}
