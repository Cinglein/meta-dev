use crate::selection::Selection;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exposure {
    Diff,
    Within,
    Outside,
    PaysDown,
}

impl Exposure {
    #[must_use]
    pub fn exposed(self, selection: &Selection) -> Option<Exposure> {
        match self {
            Exposure::Diff => selection.reaching().map(|_| self),
            Exposure::Within => Some(self),
            Exposure::Outside | Exposure::PaysDown => None,
        }
    }

    #[must_use]
    pub fn excuse(self) -> Option<&'static str> {
        match self {
            Exposure::Outside => Some("outside the linked task's division"),
            Exposure::PaysDown => Some("paid down by the linked task"),
            Exposure::Diff | Exposure::Within => None,
        }
    }
}
