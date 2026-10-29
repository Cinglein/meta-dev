#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Standing {
    Same,
    Gone,
    Changed,
}

impl Standing {
    #[must_use]
    pub fn joined(self, leader: Self) -> Self {
        match (self, leader) {
            (Self::Same, Self::Changed) => Self::Changed,
            (own, _) => own,
        }
    }
}
