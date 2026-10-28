use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(from = "bool")]
pub enum Origin {
    SameRepository,
    #[default]
    Fork,
}

impl Origin {
    #[must_use]
    pub fn same(self) -> Option<()> {
        (self == Self::SameRepository).then_some(())
    }
}

impl From<bool> for Origin {
    fn from(cross_repository: bool) -> Self {
        if cross_repository { Self::Fork } else { Self::SameRepository }
    }
}
