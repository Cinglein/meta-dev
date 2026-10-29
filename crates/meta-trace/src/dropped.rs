use std::fmt;
use std::ops::Add;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Dropped(u64);

impl Dropped {
    pub fn bytes(self) -> [u8; size_of::<u64>()] {
        self.0.to_le_bytes()
    }
}

impl From<u64> for Dropped {
    fn from(count: u64) -> Self {
        Self(count)
    }
}

impl From<[u8; size_of::<u64>()]> for Dropped {
    fn from(bytes: [u8; size_of::<u64>()]) -> Self {
        Self(u64::from_le_bytes(bytes))
    }
}

impl Add for Dropped {
    type Output = Self;

    fn add(self, more: Self) -> Self {
        Self(self.0.saturating_add(more.0))
    }
}

impl fmt::Display for Dropped {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            0 => Ok(()),
            count => writeln!(formatter, "{count} spans dropped"),
        }
    }
}
