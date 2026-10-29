#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Succeeded,
    Failing,
    Undecided,
}

impl Outcome {
    #[must_use]
    pub fn decided<Verdict>(self, succeeded: Verdict, failing: Verdict) -> Option<Verdict> {
        match self {
            Self::Succeeded => Some(succeeded),
            Self::Failing => Some(failing),
            Self::Undecided => None,
        }
    }
}
