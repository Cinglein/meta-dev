use crate::selection::Selection;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exposure {
    Diff,
    Within,
    Outside,
    CodeOutside,
    PaysDown,
}

impl Exposure {
    #[must_use]
    pub fn exposed(self, selection: &Selection) -> Option<Exposure> {
        match self {
            Exposure::Diff | Exposure::CodeOutside => selection.reaching().map(|_| self),
            Exposure::Within => Some(self),
            Exposure::Outside | Exposure::PaysDown => None,
        }
    }

    #[must_use]
    pub fn excuse(self) -> Option<&'static str> {
        match self {
            Exposure::Outside => Some("outside the linked task's division"),
            Exposure::CodeOutside => {
                Some("outside the linked task's division; a diff in its domain reaches it")
            }
            Exposure::PaysDown => Some("paid down by the linked task"),
            Exposure::Diff | Exposure::Within => None,
        }
    }
}
