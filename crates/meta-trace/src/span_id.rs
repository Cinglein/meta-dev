use std::fmt;
use std::str::FromStr;

use tracing::Id;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpanId(u64);

impl SpanId {
    const PROCESS_SHIFT: u32 = 32;
    const INDEX_MASK: u64 = 0xffff_ffff;
    const HEX: u32 = 16;

    #[must_use]
    pub fn new(pid: u32, index: usize) -> Self {
        Self(u64::from(pid) << Self::PROCESS_SHIFT).indexed(index)
    }

    #[must_use]
    pub fn indexed(self, index: usize) -> Self {
        let index = u64::try_from(index).unwrap_or(Self::INDEX_MASK).min(Self::INDEX_MASK - 1);
        Self((self.0 & !Self::INDEX_MASK) | (index + 1))
    }

    #[must_use]
    pub fn read(word: u64) -> Option<Self> {
        Some(Self(word)).filter(|span| span.0 & Self::INDEX_MASK != 0)
    }

    #[must_use]
    pub fn word(self) -> u64 {
        self.0
    }

    #[must_use]
    pub fn index(self) -> usize {
        usize::try_from((self.0 & Self::INDEX_MASK).saturating_sub(1)).unwrap_or(usize::MAX)
    }

    #[must_use]
    pub fn pid(self) -> u32 {
        u32::try_from(self.0 >> Self::PROCESS_SHIFT).unwrap_or(u32::MAX)
    }
}

impl From<SpanId> for Id {
    fn from(span: SpanId) -> Self {
        Id::from_u64(span.0)
    }
}

impl From<&Id> for SpanId {
    fn from(span: &Id) -> Self {
        Self(span.into_u64())
    }
}

impl FromStr for SpanId {
    type Err = ();

    fn from_str(text: &str) -> Result<Self, ()> {
        u64::from_str_radix(text, Self::HEX).ok().and_then(Self::read).ok_or(())
    }
}

impl fmt::Display for SpanId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:x}", self.0)
    }
}
