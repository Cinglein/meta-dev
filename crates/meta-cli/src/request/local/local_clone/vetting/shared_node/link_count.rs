#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinkCount(u64);

impl LinkCount {
    pub fn new(links: u64) -> Self {
        Self(links)
    }

    pub fn seen_all(self, seen: usize) -> bool {
        u64::try_from(seen).is_ok_and(|seen| seen >= self.0)
    }
}
