use crate::selection::Selection;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exposure {
    Diff,
    Within,
    Outside,
    PaysDown,
    Pending,
    Reported,
}

impl Exposure {
    #[must_use]
    pub fn exposed(self, selection: &Selection) -> Option<Exposure> {
        match self {
            Exposure::Diff => selection.reaching().map(|_| self),
            Exposure::Within => Some(self),
            Exposure::Outside | Exposure::PaysDown | Exposure::Pending | Exposure::Reported => None,
        }
    }

    #[must_use]
    pub fn excuse(self) -> Option<&'static str> {
        match self {
            Exposure::Outside => Some("outside the linked task's division"),
            Exposure::PaysDown => Some("paid down by the linked task"),
            Exposure::Reported => Some("reported only; this run is not a pull request"),
            Exposure::Pending => {
                Some("not in the merge-base policy; enforced from the next change")
            }
            Exposure::Diff | Exposure::Within => None,
        }
    }
}
