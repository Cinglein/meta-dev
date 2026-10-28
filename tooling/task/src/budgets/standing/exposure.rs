use crate::budgets::budget::State;
use crate::selection::Selection;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exposure {
    Diff,
    Within,
    Outside,
    PaysDown,
    Pending,
    Reported,
    Improves,
    Unchanged,
    Unmeasured,
}

impl Exposure {
    #[must_use]
    pub fn exposed(self, selection: &Selection) -> Option<Exposure> {
        match self {
            Exposure::Diff | Exposure::Unchanged => selection.reaching().map(|_| self),
            Exposure::Within | Exposure::Unmeasured => Some(self),
            Exposure::Outside
            | Exposure::PaysDown
            | Exposure::Pending
            | Exposure::Reported
            | Exposure::Improves => None,
        }
    }

    #[must_use]
    pub fn unknown_state(self) -> Option<State> {
        (self == Exposure::Unmeasured).then_some(State::Over)
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
            Exposure::Improves => Some("moved toward its line since the merge base"),
            Exposure::Unchanged => Some("unchanged since the merge base"),
            Exposure::Unmeasured => Some(
                "unmeasured at the merge base or head; fails closed until both are measured or the linked task fixes an open item in its division",
            ),
            Exposure::Diff | Exposure::Within => None,
        }
    }
}
