use std::iter::Sum;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryBytes(u64);

impl MemoryBytes {
    pub(crate) const fn new(bytes: u64) -> MemoryBytes {
        MemoryBytes(bytes)
    }

    #[must_use]
    pub const fn count(self) -> u64 {
        self.0
    }
}

impl Sum for MemoryBytes {
    fn sum<Bytes: Iterator<Item = MemoryBytes>>(bytes: Bytes) -> MemoryBytes {
        MemoryBytes(bytes.map(MemoryBytes::count).sum())
    }
}
